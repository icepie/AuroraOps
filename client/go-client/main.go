package main

import (
	"bytes"
	"context"
	"crypto/sha1"
	"embed"
	"encoding/binary"
	"encoding/hex"
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"io"
	"io/fs"
	"log"
	"net"
	"net/http"
	"os"
	"os/exec"
	"os/signal"
	"path/filepath"
	"runtime"
	"sort"
	"strconv"
	"strings"
	"sync"
	"syscall"
	"time"

	"github.com/shirou/gopsutil/v3/cpu"
	"github.com/shirou/gopsutil/v3/disk"
	"github.com/shirou/gopsutil/v3/mem"
)

type Config struct {
	ServerHost string `json:"serverHost"`
	DeviceName string `json:"deviceName"`
	HTTPBase   string `json:"httpBase"`
	DeviceID   uint64 `json:"deviceId"`
	Token      string `json:"token"`
	TCPAddress string `json:"tcpAddress"`
	Hostname   string `json:"hostname"`
}

type AgentStatus struct {
	State      string `json:"state"`
	DeviceID   uint64 `json:"deviceId,omitempty"`
	TCPAddress string `json:"tcpAddress,omitempty"`
	Message    string `json:"message,omitempty"`
	UpdatedAt  int64  `json:"updatedAt"`
}

type AgentState struct {
	mu         sync.RWMutex
	configPath string
	config     Config
	status     AgentStatus
	cancel     context.CancelFunc
	running    bool
}

type HTTPEnvelope struct {
	Code    int             `json:"code"`
	Message string          `json:"message"`
	Data    json.RawMessage `json:"data"`
}

type RegisterRequest struct {
	Name       string `json:"name"`
	Hostname   string `json:"hostname"`
	Ip         string `json:"ip"`
	DeviceType string `json:"deviceType"`
	OsName     string `json:"osName"`
	Location   string `json:"location"`
}

type RegisterResponse struct {
	ID         uint64 `json:"id"`
	Token      string `json:"token"`
	TCPAddress string `json:"tcpAddress"`
}

type HeartbeatRequest struct {
	ID       uint64 `json:"id"`
	Hostname string `json:"hostname"`
	Ip       string `json:"ip"`
	OsName   string `json:"osName"`
}

type AssetSyncRequest struct {
	DeviceID uint64       `json:"deviceId"`
	Assets   []AssetEntry `json:"assets"`
}

type AssetEntry struct {
	AssetType     string `json:"assetType"`
	UniqueKey     string `json:"uniqueKey"`
	AssetName     string `json:"assetName"`
	Brand         string `json:"brand"`
	Model         string `json:"model"`
	SerialNo      string `json:"serialNo"`
	Specification string `json:"specification"`
	Source        string `json:"source"`
	SyncHash      string `json:"syncHash"`
	Remark        string `json:"remark"`
}

type ServerAssetEntry struct {
	Id            uint64 `json:"id"`
	AssetType     string `json:"assetType"`
	UniqueKey     string `json:"uniqueKey"`
	AssetName     string `json:"assetName"`
	Brand         string `json:"brand"`
	Model         string `json:"model"`
	SerialNo      string `json:"serialNo"`
	Specification string `json:"specification"`
	Source        string `json:"source"`
	SyncHash      string `json:"syncHash"`
	Remark        string `json:"remark"`
	Status        int    `json:"status"`
	LastSeenAt    string `json:"lastSeenAt"`
}

type AssetDiagnostic struct {
	Name    string `json:"name"`
	OK      bool   `json:"ok"`
	Count   int    `json:"count"`
	Message string `json:"message,omitempty"`
}

type TCPMessage struct {
	Router  string      `json:"router"`
	TraceID string      `json:"traceId,omitempty"`
	Data    interface{} `json:"data"`
}

type TCPEnvelope struct {
	Router string          `json:"router"`
	Data   json.RawMessage `json:"data"`
}

type TCPResponse struct {
	Code    int    `json:"code"`
	Message string `json:"message"`
}

type DeviceLoginRequest struct {
	DeviceID  uint64 `json:"deviceId"`
	Name      string `json:"name"`
	Hostname  string `json:"hostname"`
	Token     string `json:"token"`
	Timestamp int64  `json:"timestamp"`
}

type DeviceHeartbeatRequest struct {
	DeviceID uint64 `json:"deviceId"`
}

type SaveConfigPayload struct {
	ServerHost string `json:"serverHost"`
	DeviceName string `json:"deviceName"`
}

type ControlResponse struct {
	OK           bool               `json:"ok"`
	Status       AgentStatus        `json:"status"`
	Config       Config             `json:"config"`
	Assets       []AssetEntry       `json:"assets,omitempty"`
	ServerAssets []ServerAssetEntry `json:"serverAssets,omitempty"`
	Diagnostics  []AssetDiagnostic  `json:"diagnostics,omitempty"`
	Message      string             `json:"message,omitempty"`
}

type HeadlessConnectPayload struct {
	ServerHost string `json:"serverHost"`
	DeviceName string `json:"deviceName"`
}

type RunOptions struct {
	ConfigPath string
	Port       int
	Headless   bool
	ServerHost string
	DeviceName string
	ServiceCmd string
}

//go:embed ui/*
var uiFS embed.FS

func main() {
	var (
		configPath = flag.String("config", "", "config file path")
		port       = flag.Int("port", 18765, "local http port")
		headless   = flag.Bool("headless", false, "run without electron ui")
		serverHost = flag.String("server", "", "headless server host")
		deviceName = flag.String("name", "", "headless device name")
		serviceCmd = flag.String("service", "", "windows service command: install|uninstall|run")
	)
	flag.Parse()

	path, err := resolveConfigPath(*configPath)
	if err != nil {
		log.Fatal(err)
	}

	options := RunOptions{
		ConfigPath: path,
		Port:       *port,
		Headless:   *headless,
		ServerHost: strings.TrimSpace(*serverHost),
		DeviceName: strings.TrimSpace(*deviceName),
		ServiceCmd: strings.TrimSpace(*serviceCmd),
	}

	if err = runMain(options); err != nil {
		log.Fatal(err)
	}
}

func runMain(options RunOptions) error {
	if options.ServiceCmd != "" {
		return handleServiceCommand(options)
	}
	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()
	return runAgent(ctx, options)
}

func runAgent(ctx context.Context, options RunOptions) error {
	state := &AgentState{
		configPath: options.ConfigPath,
		status: AgentStatus{
			State:     "idle",
			UpdatedAt: time.Now().UnixMilli(),
		},
	}
	_ = state.loadConfig()

	if options.ServerHost != "" && options.DeviceName != "" {
		if _, err := state.saveConfig(options.ServerHost, options.DeviceName); err != nil {
			return err
		}
	}

	server := &http.Server{
		Addr:              fmt.Sprintf("0.0.0.0:%d", options.Port),
		Handler:           state.routes(),
		ReadHeaderTimeout: 10 * time.Second,
	}

	serverErr := make(chan error, 1)
	go func() {
		if err := server.ListenAndServe(); err != nil && !errors.Is(err, http.ErrServerClosed) {
			serverErr <- err
			return
		}
		serverErr <- nil
	}()

	if options.Headless {
		log.Printf("AuroraOps agent ui available at http://127.0.0.1:%d/\n", options.Port)
		if state.getConfig().ServerHost != "" && state.getConfig().DeviceName != "" {
			if _, err := state.startAgent(); err != nil {
				log.Printf("headless auto-start failed: %v\n", err)
			}
		}
	}

	select {
	case <-ctx.Done():
	case err := <-serverErr:
		if err != nil {
			return fmt.Errorf("local http server failed: %w", err)
		}
	}

	shutdownCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	state.stopAgent()
	return server.Shutdown(shutdownCtx)
}

func (s *AgentState) routes() http.Handler {
	mux := http.NewServeMux()
	mux.HandleFunc("/api/status", s.handleGetStatus)
	mux.HandleFunc("/api/config", s.handleSaveConfig)
	mux.HandleFunc("/api/assets/preview", s.handlePreviewAssets)
	mux.HandleFunc("/api/start", s.handleStart)
	mux.HandleFunc("/api/stop", s.handleStop)
	uiSub, _ := fs.Sub(uiFS, "ui")
	mux.Handle("/", http.FileServer(http.FS(uiSub)))
	return mux
}

func (s *AgentState) handleGetStatus(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		writeJSON(w, http.StatusMethodNotAllowed, ControlResponse{OK: false, Message: "method not allowed"})
		return
	}
	writeJSON(w, http.StatusOK, ControlResponse{
		OK:     true,
		Status: s.getStatus(),
		Config: s.getConfig(),
	})
}

func (s *AgentState) handleSaveConfig(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		writeJSON(w, http.StatusMethodNotAllowed, ControlResponse{OK: false, Message: "method not allowed"})
		return
	}
	var payload SaveConfigPayload
	if err := json.NewDecoder(r.Body).Decode(&payload); err != nil {
		writeJSON(w, http.StatusBadRequest, ControlResponse{OK: false, Message: err.Error()})
		return
	}
	cfg, err := s.saveConfig(payload.ServerHost, payload.DeviceName)
	if err != nil {
		writeJSON(w, http.StatusBadRequest, ControlResponse{OK: false, Message: err.Error()})
		return
	}
	writeJSON(w, http.StatusOK, ControlResponse{
		OK:     true,
		Status: s.getStatus(),
		Config: cfg,
	})
}

func (s *AgentState) handlePreviewAssets(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet {
		writeJSON(w, http.StatusMethodNotAllowed, ControlResponse{OK: false, Message: "method not allowed"})
		return
	}
	cfg := s.getConfig()
	assets, diagnostics, err := collectAssetsWithDiagnostics()
	if err != nil {
		writeJSON(w, http.StatusBadRequest, ControlResponse{
			OK:          false,
			Status:      s.getStatus(),
			Config:      cfg,
			Diagnostics: diagnostics,
			Message:     err.Error(),
		})
		return
	}
	serverAssets := []ServerAssetEntry{}
	if cfg.DeviceID > 0 && strings.TrimSpace(cfg.HTTPBase) != "" {
		if pulled, pullErr := s.pullServerAssets(r.Context(), cfg); pullErr == nil {
			serverAssets = pulled
		}
	}
	writeJSON(w, http.StatusOK, ControlResponse{
		OK:           true,
		Status:       s.getStatus(),
		Config:       cfg,
		Assets:       assets,
		ServerAssets: serverAssets,
		Diagnostics:  diagnostics,
	})
}

func (s *AgentState) handleStart(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		writeJSON(w, http.StatusMethodNotAllowed, ControlResponse{OK: false, Message: "method not allowed"})
		return
	}
	status, err := s.startAgent()
	if err != nil {
		writeJSON(w, http.StatusBadRequest, ControlResponse{
			OK:      false,
			Status:  s.getStatus(),
			Config:  s.getConfig(),
			Message: err.Error(),
		})
		return
	}
	writeJSON(w, http.StatusOK, ControlResponse{
		OK:     true,
		Status: status,
		Config: s.getConfig(),
	})
}

func (s *AgentState) handleStop(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		writeJSON(w, http.StatusMethodNotAllowed, ControlResponse{OK: false, Message: "method not allowed"})
		return
	}
	status := s.stopAgent()
	writeJSON(w, http.StatusOK, ControlResponse{
		OK:     true,
		Status: status,
		Config: s.getConfig(),
	})
}

func (s *AgentState) startAgent() (AgentStatus, error) {
	s.mu.Lock()
	defer s.mu.Unlock()

	if s.running {
		return s.status, nil
	}
	if strings.TrimSpace(s.config.ServerHost) == "" || strings.TrimSpace(s.config.DeviceName) == "" {
		return s.status, errors.New("serverHost and deviceName are required")
	}

	ctx, cancel := context.WithCancel(context.Background())
	s.cancel = cancel
	s.running = true
	s.status = AgentStatus{
		State:     "starting",
		Message:   "agent starting",
		UpdatedAt: time.Now().UnixMilli(),
	}

	cfg := s.config
	go s.runConnector(ctx, cfg)
	return s.status, nil
}

func (s *AgentState) stopAgent() AgentStatus {
	s.mu.Lock()
	defer s.mu.Unlock()

	if s.cancel != nil {
		s.cancel()
	}
	s.cancel = nil
	s.running = false
	s.status = AgentStatus{
		State:      "stopped",
		DeviceID:   s.config.DeviceID,
		TCPAddress: s.config.TCPAddress,
		Message:    "agent stopped",
		UpdatedAt:  time.Now().UnixMilli(),
	}
	return s.status
}

func (s *AgentState) runConnector(ctx context.Context, cfg Config) {
	httpClient := &http.Client{Timeout: 15 * time.Second}
	hostname, _ := os.Hostname()
	ip := detectIP()
	osName := runtime.GOOS

	updatedCfg, err := registerDevice(ctx, httpClient, s.configPath, cfg, hostname, ip, osName)
	if err != nil {
		s.updateStatus("error", updatedCfg.DeviceID, updatedCfg.TCPAddress, err.Error())
		s.markStopped()
		return
	}
	s.setConfig(updatedCfg)
	s.updateStatus("registered", updatedCfg.DeviceID, updatedCfg.TCPAddress, "")

	if err = syncAssets(ctx, httpClient, updatedCfg); err != nil {
		log.Printf("asset sync failed after registration: %v\n", err)
		s.updateStatus("registered", updatedCfg.DeviceID, updatedCfg.TCPAddress, "资产同步失败: "+err.Error())
	}

	backoff := 2 * time.Second
	for {
		if ctx.Err() != nil {
			s.markStopped()
			return
		}
		err = connectTCP(ctx, updatedCfg)
		if ctx.Err() != nil {
			s.markStopped()
			return
		}
		s.updateStatus("reconnecting", updatedCfg.DeviceID, updatedCfg.TCPAddress, errString(err))
		select {
		case <-ctx.Done():
			s.markStopped()
			return
		case <-time.After(backoff):
		}
		if backoff < 30*time.Second {
			backoff *= 2
		}
	}
}

func registerDevice(ctx context.Context, client *http.Client, cfgPath string, cfg Config, hostname, ip, osName string) (Config, error) {
	req := RegisterRequest{
		Name:       cfg.DeviceName,
		Hostname:   hostname,
		Ip:         ip,
		DeviceType: "physical",
		OsName:     osName,
		Location:   runtime.GOARCH,
	}
	var reg RegisterResponse
	if err := postJSON(ctx, client, cfg.HTTPBase+"/admin/client/register", req, &reg); err != nil {
		return cfg, err
	}
	cfg.DeviceID = reg.ID
	cfg.Token = reg.Token
	if reg.TCPAddress != "" {
		cfg.TCPAddress = reg.TCPAddress
	}
	cfg.Hostname = hostname
	if err := saveConfigFile(cfgPath, cfg); err != nil {
		return cfg, err
	}
	return cfg, nil
}

func connectTCP(ctx context.Context, cfg Config) error {
	address := cfg.TCPAddress
	if address == "" {
		return errors.New("missing tcp address")
	}
	conn, err := net.DialTimeout("tcp", address, 10*time.Second)
	if err != nil {
		return err
	}
	defer conn.Close()

	if err := sendTCP(conn, "DeviceLoginReq", DeviceLoginRequest{
		DeviceID:  cfg.DeviceID,
		Name:      cfg.DeviceName,
		Hostname:  cfg.Hostname,
		Token:     cfg.Token,
		Timestamp: time.Now().Unix(),
	}); err != nil {
		return err
	}
	if err := readTCPResponse(conn, "DeviceLoginRes"); err != nil {
		return err
	}

	hbTicker := time.NewTicker(30 * time.Second)
	defer hbTicker.Stop()
	httpTicker := time.NewTicker(60 * time.Second)
	defer httpTicker.Stop()
	client := &http.Client{Timeout: 15 * time.Second}

	for {
		select {
		case <-ctx.Done():
			return nil
		case <-hbTicker.C:
			if err := sendTCP(conn, "DeviceHeartbeatReq", DeviceHeartbeatRequest{DeviceID: cfg.DeviceID}); err != nil {
				return err
			}
			if err := readTCPResponse(conn, "DeviceHeartbeatRes"); err != nil {
				return err
			}
			if statusErr := postJSON(ctx, client, cfg.HTTPBase+"/admin/client/heartbeat", HeartbeatRequest{
				ID:       cfg.DeviceID,
				Hostname: cfg.Hostname,
				Ip:       detectIP(),
				OsName:   runtime.GOOS,
			}, nil); statusErr != nil {
				log.Printf("http heartbeat failed: %v\n", statusErr)
			}
		case <-httpTicker.C:
			if err := syncAssets(ctx, client, cfg); err != nil {
				log.Printf("periodic asset sync failed: %v\n", err)
			}
		}
	}
}

func syncAssets(ctx context.Context, client *http.Client, cfg Config) error {
	assets, diagnostics, err := collectAssetsWithDiagnostics()
	if err != nil {
		return fmt.Errorf("collect assets: %w", err)
	}
	assets = normalizeAssets(assets)
	if len(assets) == 0 {
		return errors.New("collect assets: no assets returned")
	}
	if err = postJSON(ctx, client, cfg.HTTPBase+"/admin/client/assets/sync", AssetSyncRequest{
		DeviceID: cfg.DeviceID,
		Assets:   assets,
	}, nil); err != nil {
		return fmt.Errorf("push assets: %w", err)
	}
	for _, item := range diagnostics {
		if item.OK {
			log.Printf("asset diagnostic %s ok count=%d %s\n", item.Name, item.Count, item.Message)
		} else {
			log.Printf("asset diagnostic %s failed %s\n", item.Name, item.Message)
		}
	}
	log.Printf("asset sync success: %d assets\n", len(assets))
	return nil
}

func collectAssets() ([]AssetEntry, error) {
	assets, _, err := collectAssetsWithDiagnostics()
	return assets, err
}

func collectAssetsWithDiagnostics() ([]AssetEntry, []AssetDiagnostic, error) {
	var diagnostics []AssetDiagnostic
	if runtime.GOOS == "linux" {
		assets, linuxDiagnostics, err := collectLinuxAssets()
		diagnostics = append(diagnostics, linuxDiagnostics...)
		if err == nil && len(assets) > 0 {
			return assets, diagnostics, nil
		}
		if err != nil {
			log.Printf("collect linux assets failed, fallback to gopsutil: %v\n", err)
			diagnostics = append(diagnostics, AssetDiagnostic{
				Name:    "linux",
				OK:      false,
				Message: err.Error(),
			})
		}
	}

	var assets []AssetEntry

	cpuInfo, err := cpu.Info()
	if err == nil {
		seenCPU := make(map[string]struct{})
		for _, item := range cpuInfo {
			key := strings.TrimSpace(item.PhysicalID)
			if key == "" {
				key = sanitize(item.VendorID + "-" + item.ModelName)
			}
			if _, ok := seenCPU[key]; ok {
				continue
			}
			seenCPU[key] = struct{}{}
			spec := strings.TrimSpace(fmt.Sprintf("%d cores %.2fMhz", item.Cores, item.Mhz))
			assets = append(assets, AssetEntry{
				AssetType:     "cpu",
				UniqueKey:     key,
				AssetName:     firstNonEmpty(item.ModelName, item.Model),
				Brand:         item.VendorID,
				Model:         firstNonEmpty(item.ModelName, item.Model),
				SerialNo:      item.PhysicalID,
				Specification: spec,
				Remark:        "auto:agent",
			})
		}
		diagnostics = append(diagnostics, AssetDiagnostic{
			Name:    "fallback-cpu",
			OK:      len(assets) > 0,
			Count:   len(assets),
			Message: "gopsutil cpu fallback",
		})
	} else {
		diagnostics = append(diagnostics, AssetDiagnostic{
			Name:    "fallback-cpu",
			OK:      false,
			Message: err.Error(),
		})
	}

	vmem, err := mem.VirtualMemory()
	if err == nil {
		assets = append(assets, AssetEntry{
			AssetType:     "memory",
			UniqueKey:     fmt.Sprintf("memory-%d", vmem.Total),
			AssetName:     "System Memory",
			Model:         "RAM",
			Specification: fmt.Sprintf("%d bytes", vmem.Total),
			Remark:        "auto:agent",
		})
		diagnostics = append(diagnostics, AssetDiagnostic{
			Name:    "fallback-memory",
			OK:      true,
			Count:   1,
			Message: "gopsutil memory fallback",
		})
	} else {
		diagnostics = append(diagnostics, AssetDiagnostic{
			Name:    "fallback-memory",
			OK:      false,
			Message: err.Error(),
		})
	}

	partitions, err := disk.Partitions(false)
	if err == nil {
		diskCount := 0
		seenDisk := make(map[string]struct{})
		for _, part := range partitions {
			key := sanitize(part.Device)
			if key == "" {
				key = sanitize(part.Device + "-" + part.Mountpoint)
			}
			if _, ok := seenDisk[key]; ok {
				continue
			}
			seenDisk[key] = struct{}{}
			usage, usageErr := disk.Usage(part.Mountpoint)
			if usageErr != nil {
				continue
			}
			assets = append(assets, AssetEntry{
				AssetType:     "disk",
				UniqueKey:     key,
				AssetName:     part.Device,
				Model:         part.Fstype,
				Specification: fmt.Sprintf("%s %d bytes", part.Mountpoint, usage.Total),
				Remark:        "auto:agent",
			})
			diskCount++
		}
		diagnostics = append(diagnostics, AssetDiagnostic{
			Name:    "fallback-disk",
			OK:      diskCount > 0,
			Count:   diskCount,
			Message: "gopsutil disk fallback",
		})
	} else {
		diagnostics = append(diagnostics, AssetDiagnostic{
			Name:    "fallback-disk",
			OK:      false,
			Message: err.Error(),
		})
	}
	if len(assets) == 0 {
		return nil, diagnostics, errors.New("no assets collected")
	}
	return finalizeAssets(assets), diagnostics, nil
}

func normalizeAssets(items []AssetEntry) []AssetEntry {
	result := make([]AssetEntry, 0, len(items))
	for _, item := range items {
		item.AssetType = strings.TrimSpace(item.AssetType)
		originalName := strings.TrimSpace(item.AssetName)
		item.AssetName = firstNonEmpty(item.AssetName, item.Model, item.Brand, defaultAssetName(item.AssetType))
		item.Model = strings.TrimSpace(item.Model)
		item.Brand = strings.TrimSpace(item.Brand)
		item.SerialNo = strings.TrimSpace(item.SerialNo)
		item.UniqueKey = firstNonEmpty(strings.TrimSpace(item.UniqueKey), sanitize(item.AssetType+"-"+item.AssetName+"-"+item.Model+"-"+item.SerialNo))
		item.Specification = strings.TrimSpace(item.Specification)
		if originalName == "" {
			item.Remark = strings.Join(filterEmpty([]string{item.Remark, "normalized:assetName"}), " | ")
		}
		result = append(result, item)
	}
	return result
}

func defaultAssetName(assetType string) string {
	switch strings.TrimSpace(assetType) {
	case "cpu":
		return "CPU"
	case "memory":
		return "System Memory"
	case "disk":
		return "Disk"
	case "network":
		return "网卡"
	case "motherboard":
		return "Motherboard"
	case "bios":
		return "BIOS"
	default:
		return "Unknown Asset"
	}
}

type linuxLscpuPayload struct {
	Lscpu []struct {
		Field string `json:"field"`
		Data  string `json:"data"`
	} `json:"lscpu"`
}

type linuxLsblkPayload struct {
	Blockdevices []linuxBlockDevice `json:"blockdevices"`
}

type linuxBlockDevice struct {
	Name     string             `json:"name"`
	Type     string             `json:"type"`
	Size     string             `json:"size"`
	Model    string             `json:"model"`
	Vendor   string             `json:"vendor"`
	Serial   string             `json:"serial"`
	Path     string             `json:"path"`
	Rota     bool               `json:"rota"`
	Tran     string             `json:"tran"`
	Children []linuxBlockDevice `json:"children"`
}

type linuxLshwNode struct {
	ID            string          `json:"id"`
	Class         string          `json:"class"`
	Description   string          `json:"description"`
	Product       string          `json:"product"`
	Vendor        string          `json:"vendor"`
	Version       string          `json:"version"`
	Serial        string          `json:"serial"`
	LogicalName   any             `json:"logicalname"`
	BusInfo       string          `json:"businfo"`
	PhysID        string          `json:"physid"`
	Size          any             `json:"size"`
	Capacity      any             `json:"capacity"`
	Units         string          `json:"units"`
	Configuration map[string]any  `json:"configuration"`
	Children      []linuxLshwNode `json:"children"`
}

type linuxIPLink struct {
	IfName   string   `json:"ifname"`
	LinkType string   `json:"link_type"`
	Address  string   `json:"address"`
	Master   string   `json:"master"`
	Flags    []string `json:"flags"`
}

func collectLinuxAssets() ([]AssetEntry, []AssetDiagnostic, error) {
	var assets []AssetEntry
	var diagnostics []AssetDiagnostic

	if cpuAsset, err := collectLinuxCPUAsset(); err == nil && cpuAsset.UniqueKey != "" {
		assets = append(assets, cpuAsset)
		diagnostics = append(diagnostics, AssetDiagnostic{Name: "cpu", OK: true, Count: 1})
	} else if err != nil {
		log.Printf("collect linux cpu failed: %v\n", err)
		diagnostics = append(diagnostics, AssetDiagnostic{Name: "cpu", OK: false, Message: err.Error()})
	} else {
		diagnostics = append(diagnostics, AssetDiagnostic{Name: "cpu", OK: false, Message: "no cpu asset returned"})
	}

	if memoryAsset, err := collectLinuxMemoryAsset(); err == nil && memoryAsset.UniqueKey != "" {
		assets = append(assets, memoryAsset)
		diagnostics = append(diagnostics, AssetDiagnostic{Name: "memory", OK: true, Count: 1})
	} else if err != nil {
		log.Printf("collect linux memory failed: %v\n", err)
		diagnostics = append(diagnostics, AssetDiagnostic{Name: "memory", OK: false, Message: err.Error()})
	} else {
		diagnostics = append(diagnostics, AssetDiagnostic{Name: "memory", OK: false, Message: "no memory asset returned"})
	}

	if boardAssets, err := collectLinuxBoardAssets(); err == nil {
		assets = append(assets, boardAssets...)
		diagnostics = append(diagnostics, AssetDiagnostic{Name: "board", OK: len(boardAssets) > 0, Count: len(boardAssets)})
	} else if err != nil {
		log.Printf("collect linux board failed: %v\n", err)
		diagnostics = append(diagnostics, AssetDiagnostic{Name: "board", OK: false, Message: err.Error()})
	}

	diskAssets, err := collectLinuxDiskAssets()
	if err == nil {
		assets = append(assets, diskAssets...)
		diagnostics = append(diagnostics, AssetDiagnostic{Name: "disk", OK: len(diskAssets) > 0, Count: len(diskAssets)})
	} else {
		log.Printf("collect linux disk failed: %v\n", err)
		diagnostics = append(diagnostics, AssetDiagnostic{Name: "disk", OK: false, Message: err.Error()})
	}

	networkAssets, err := collectLinuxNetworkAssets()
	if err == nil {
		assets = append(assets, networkAssets...)
		diagnostics = append(diagnostics, AssetDiagnostic{Name: "network", OK: len(networkAssets) > 0, Count: len(networkAssets)})
	} else {
		log.Printf("collect linux network failed: %v\n", err)
		diagnostics = append(diagnostics, AssetDiagnostic{Name: "network", OK: false, Message: err.Error()})
	}

	if len(assets) == 0 {
		return nil, diagnostics, errors.New("no linux assets collected")
	}
	return finalizeAssets(dedupeAssets(assets)), diagnostics, nil
}

func collectLinuxCPUAsset() (AssetEntry, error) {
	output, err := runCommand("lscpu", "-J")
	if err != nil {
		return AssetEntry{}, err
	}
	var payload linuxLscpuPayload
	if err = json.Unmarshal(output, &payload); err != nil {
		return AssetEntry{}, err
	}

	fields := make(map[string]string)
	for _, item := range payload.Lscpu {
		key := strings.TrimSpace(item.Field)
		fields[key] = strings.TrimSpace(item.Data)
		fields[strings.TrimSuffix(key, ":")] = strings.TrimSpace(item.Data)
	}

	model := firstNonEmpty(fields["Model name:"], fields["Model name"])
	vendor := firstNonEmpty(fields["Vendor ID:"], fields["Vendor ID"])
	sockets := parseInt(fields["Socket(s):"])
	cores := parseInt(fields["Core(s) per socket:"])
	threads := parseInt(fields["Thread(s) per core:"])
	maxMHz := firstNonEmpty(fields["CPU max MHz:"], fields["CPU max MHz"])

	specParts := make([]string, 0, 4)
	if sockets > 0 {
		specParts = append(specParts, fmt.Sprintf("%d socket(s)", sockets))
	}
	if cores > 0 {
		specParts = append(specParts, fmt.Sprintf("%d core(s)/socket", cores))
	}
	if threads > 0 {
		specParts = append(specParts, fmt.Sprintf("%d thread(s)/core", threads))
	}
	if maxMHz != "" {
		specParts = append(specParts, fmt.Sprintf("max %s MHz", trimDecimal(maxMHz)))
	}

	return AssetEntry{
		AssetType:     "cpu",
		UniqueKey:     sanitize(firstNonEmpty(fields["Model name:"], vendor, "cpu")),
		AssetName:     firstNonEmpty(model, vendor, "CPU"),
		Brand:         vendor,
		Model:         firstNonEmpty(model, "CPU"),
		Specification: strings.Join(specParts, ", "),
		Remark:        "auto:agent",
	}, nil
}

func collectLinuxMemoryAsset() (AssetEntry, error) {
	output, err := runCommand("lshw", "-json", "-class", "memory")
	if err != nil {
		vmem, memErr := mem.VirtualMemory()
		if memErr != nil {
			return AssetEntry{}, err
		}
		return AssetEntry{
			AssetType:     "memory",
			UniqueKey:     fmt.Sprintf("memory-%d", vmem.Total),
			AssetName:     "System Memory",
			Model:         "RAM",
			Specification: fmt.Sprintf("%d bytes", vmem.Total),
			Remark:        "auto:agent",
		}, nil
	}

	nodes, err := parseLshwNodes(output)
	if err != nil {
		return AssetEntry{}, err
	}

	total := uint64(0)
	for _, node := range nodes {
		if node.Class != "memory" {
			continue
		}
		if !strings.EqualFold(strings.TrimSpace(node.Description), "System memory") {
			continue
		}
		total = firstUint64(total, toUint64(node.Size), toUint64(node.Capacity))
	}
	if total == 0 {
		vmem, memErr := mem.VirtualMemory()
		if memErr == nil {
			total = vmem.Total
		}
	}
	if total == 0 {
		return AssetEntry{}, errors.New("linux memory not found")
	}

	return AssetEntry{
		AssetType:     "memory",
		UniqueKey:     fmt.Sprintf("memory-%d", total),
		AssetName:     "System Memory",
		Model:         "RAM",
		Specification: humanBytes(total),
		Remark:        "auto:agent",
	}, nil
}

func collectLinuxDiskAssets() ([]AssetEntry, error) {
	output, err := runCommand("lsblk", "-J", "-d", "-o", "NAME,TYPE,SIZE,MODEL,VENDOR,SERIAL,PATH,ROTA,TRAN")
	if err != nil {
		return nil, err
	}
	var payload linuxLsblkPayload
	if err = json.Unmarshal(output, &payload); err != nil {
		return nil, err
	}

	var assets []AssetEntry
	for _, item := range payload.Blockdevices {
		if item.Type != "disk" {
			continue
		}
		name := firstNonEmpty(item.Path, item.Name)
		model := strings.TrimSpace(item.Model)
		vendor := strings.TrimSpace(item.Vendor)
		serial := strings.TrimSpace(item.Serial)
		specParts := []string{}
		if item.Size != "" {
			specParts = append(specParts, item.Size)
		}
		if item.Tran != "" {
			specParts = append(specParts, item.Tran)
		}
		specParts = append(specParts, map[bool]string{true: "HDD", false: "SSD/NVMe"}[item.Rota])

		assets = append(assets, AssetEntry{
			AssetType:     "disk",
			UniqueKey:     sanitize(firstNonEmpty(serial, name)),
			AssetName:     name,
			Brand:         vendor,
			Model:         model,
			SerialNo:      serial,
			Specification: strings.Join(filterEmpty(specParts), ", "),
			Remark:        "auto:agent",
		})
	}
	return assets, nil
}

func collectLinuxBoardAssets() ([]AssetEntry, error) {
	output, err := runCommand("dmidecode", "-t", "baseboard", "-t", "bios")
	if err != nil {
		return nil, nil
	}

	sections := splitDMISections(string(output))
	var assets []AssetEntry
	for _, section := range sections {
		sectionType := strings.ToLower(strings.TrimSpace(section["type"]))
		switch sectionType {
		case "base board information":
			manufacturer := firstNonEmpty(section["Manufacturer"], section["manufacturer"])
			product := firstNonEmpty(section["Product Name"], section["product name"])
			serial := firstNonEmpty(section["Serial Number"], section["serial number"])
			version := firstNonEmpty(section["Version"], section["version"])
			assets = append(assets, AssetEntry{
				AssetType:     "motherboard",
				UniqueKey:     sanitize(firstNonEmpty(serial, manufacturer+"-"+product, "motherboard")),
				AssetName:     firstNonEmpty(product, "Motherboard"),
				Brand:         manufacturer,
				Model:         product,
				SerialNo:      serial,
				Specification: version,
				Remark:        "auto:agent",
			})
		case "bios information":
			vendor := firstNonEmpty(section["Vendor"], section["vendor"])
			version := firstNonEmpty(section["Version"], section["version"])
			releaseDate := firstNonEmpty(section["Release Date"], section["release date"])
			assets = append(assets, AssetEntry{
				AssetType:     "bios",
				UniqueKey:     sanitize(firstNonEmpty(vendor+"-"+version, "bios")),
				AssetName:     "BIOS",
				Brand:         vendor,
				Model:         version,
				Specification: strings.Join(filterEmpty([]string{version, releaseDate}), ", "),
				Remark:        "auto:agent",
			})
		}
	}
	return assets, nil
}

func collectLinuxNetworkAssets() ([]AssetEntry, error) {
	output, err := runCommand("lshw", "-json", "-class", "network")
	if err != nil {
		return nil, err
	}
	nodes, err := parseLshwNodes(output)
	if err != nil {
		return nil, err
	}

	ipOutput, _ := runCommand("ip", "-j", "link")
	linkMap := make(map[string]linuxIPLink)
	if len(ipOutput) > 0 {
		var links []linuxIPLink
		if json.Unmarshal(ipOutput, &links) == nil {
			for _, item := range links {
				linkMap[item.IfName] = item
			}
		}
	}

	var assets []AssetEntry
	for _, node := range nodes {
		if node.Class != "network" {
			continue
		}
		logicalName := normalizeLogicalName(node.LogicalName)
		if logicalName == "" || isVirtualInterface(logicalName, linkMap[logicalName]) {
			continue
		}
		driver := ""
		speed := ""
		if node.Configuration != nil {
			driver = toString(node.Configuration["driver"])
			speed = firstNonEmpty(toString(node.Configuration["speed"]), toString(node.Size))
		}
		assets = append(assets, AssetEntry{
			AssetType:     "network",
			UniqueKey:     sanitize(firstNonEmpty(node.Serial, logicalName, node.BusInfo)),
			AssetName:     firstNonEmpty(logicalName, node.Description, "网卡"),
			Brand:         strings.TrimSpace(node.Vendor),
			Model:         firstNonEmpty(node.Product, node.Description),
			SerialNo:      strings.TrimSpace(node.Serial),
			Specification: strings.Join(filterEmpty([]string{node.Description, driver, speed}), ", "),
			Remark:        strings.Join(filterEmpty([]string{"auto:agent", strings.TrimSpace(node.BusInfo)}), " | "),
		})
	}
	return assets, nil
}

func parseLshwNodes(output []byte) ([]linuxLshwNode, error) {
	var list []linuxLshwNode
	if err := json.Unmarshal(output, &list); err == nil {
		return flattenLshwNodes(list), nil
	}
	var single linuxLshwNode
	if err := json.Unmarshal(output, &single); err != nil {
		return nil, err
	}
	return flattenLshwNodes([]linuxLshwNode{single}), nil
}

func flattenLshwNodes(nodes []linuxLshwNode) []linuxLshwNode {
	var result []linuxLshwNode
	var walk func(items []linuxLshwNode)
	walk = func(items []linuxLshwNode) {
		for _, item := range items {
			result = append(result, item)
			if len(item.Children) > 0 {
				walk(item.Children)
			}
		}
	}
	walk(nodes)
	return result
}

func runCommand(name string, args ...string) ([]byte, error) {
	cmd := exec.Command(name, args...)
	cmd.Env = append(os.Environ(), "LC_ALL=C", "LANG=C")
	return cmd.Output()
}

func splitDMISections(output string) []map[string]string {
	var sections []map[string]string
	var current map[string]string
	lines := strings.Split(output, "\n")
	for _, raw := range lines {
		line := strings.TrimRight(raw, "\r")
		if strings.TrimSpace(line) == "" {
			continue
		}
		if !strings.HasPrefix(line, "\t") && !strings.HasPrefix(line, " ") {
			if current != nil && len(current) > 0 {
				sections = append(sections, current)
			}
			current = map[string]string{"type": strings.TrimSpace(line)}
			continue
		}
		if current == nil {
			continue
		}
		trimmed := strings.TrimSpace(line)
		parts := strings.SplitN(trimmed, ":", 2)
		if len(parts) != 2 {
			continue
		}
		current[strings.TrimSpace(parts[0])] = strings.TrimSpace(parts[1])
	}
	if current != nil && len(current) > 0 {
		sections = append(sections, current)
	}
	return sections
}

func normalizeLogicalName(value any) string {
	switch v := value.(type) {
	case string:
		return strings.TrimSpace(v)
	case []any:
		for _, item := range v {
			if text, ok := item.(string); ok && strings.TrimSpace(text) != "" {
				return strings.TrimSpace(text)
			}
		}
	case []string:
		for _, item := range v {
			if strings.TrimSpace(item) != "" {
				return strings.TrimSpace(item)
			}
		}
	}
	return ""
}

func isVirtualInterface(name string, link linuxIPLink) bool {
	name = strings.TrimSpace(name)
	if name == "" {
		return true
	}
	virtualPrefixes := []string{"lo", "docker", "br-", "veth", "vmnet", "virbr", "tun", "tap", "zt", "tailscale", "cni", "flannel", "kube", "home"}
	for _, prefix := range virtualPrefixes {
		if strings.HasPrefix(name, prefix) {
			return true
		}
	}
	if link.Master != "" {
		return true
	}
	return link.LinkType != "" && link.LinkType != "ether"
}

func toString(value any) string {
	switch v := value.(type) {
	case string:
		return strings.TrimSpace(v)
	case float64:
		return trimDecimal(strconv.FormatFloat(v, 'f', -1, 64))
	case int64:
		return strconv.FormatInt(v, 10)
	case int:
		return strconv.Itoa(v)
	default:
		return ""
	}
}

func dedupeAssets(items []AssetEntry) []AssetEntry {
	seen := make(map[string]AssetEntry)
	order := make([]string, 0, len(items))
	for _, item := range items {
		key := item.AssetType + "::" + item.UniqueKey
		if key == item.AssetType+"::" {
			key = item.AssetType + "::" + sanitize(item.AssetName+item.Model+item.SerialNo)
		}
		if _, ok := seen[key]; ok {
			continue
		}
		seen[key] = item
		order = append(order, key)
	}
	sort.Strings(order)
	result := make([]AssetEntry, 0, len(order))
	for _, key := range order {
		result = append(result, seen[key])
	}
	return result
}

func finalizeAssets(items []AssetEntry) []AssetEntry {
	result := make([]AssetEntry, 0, len(items))
	for _, item := range items {
		if strings.TrimSpace(item.Source) == "" {
			item.Source = "agent"
		}
		item.SyncHash = assetSyncHash(item)
		result = append(result, item)
	}
	return result
}

func assetSyncHash(item AssetEntry) string {
	raw := strings.Join([]string{
		strings.TrimSpace(item.AssetType),
		strings.TrimSpace(item.UniqueKey),
		strings.TrimSpace(item.AssetName),
		strings.TrimSpace(item.Brand),
		strings.TrimSpace(item.Model),
		strings.TrimSpace(item.Specification),
	}, "|")
	sum := sha1.Sum([]byte(raw))
	return hex.EncodeToString(sum[:])
}

func firstNonEmpty(values ...string) string {
	for _, value := range values {
		value = strings.TrimSpace(value)
		if value != "" {
			return value
		}
	}
	return ""
}

func filterEmpty(values []string) []string {
	result := make([]string, 0, len(values))
	for _, value := range values {
		value = strings.TrimSpace(value)
		if value != "" {
			result = append(result, value)
		}
	}
	return result
}

func parseInt(value string) int {
	value = strings.TrimSpace(value)
	if value == "" {
		return 0
	}
	number, _ := strconv.Atoi(value)
	return number
}

func trimDecimal(value string) string {
	value = strings.TrimSpace(value)
	if value == "" {
		return ""
	}
	if strings.Contains(value, ".") {
		value = strings.TrimRight(strings.TrimRight(value, "0"), ".")
	}
	return value
}

func toUint64(value any) uint64 {
	switch v := value.(type) {
	case float64:
		return uint64(v)
	case int:
		return uint64(v)
	case int64:
		return uint64(v)
	case uint64:
		return v
	case json.Number:
		n, _ := v.Int64()
		return uint64(n)
	case string:
		n, _ := strconv.ParseUint(strings.TrimSpace(v), 10, 64)
		return n
	default:
		return 0
	}
}

func firstUint64(values ...uint64) uint64 {
	for _, value := range values {
		if value > 0 {
			return value
		}
	}
	return 0
}

func humanBytes(size uint64) string {
	units := []string{"B", "KB", "MB", "GB", "TB", "PB"}
	value := float64(size)
	unit := 0
	for value >= 1024 && unit < len(units)-1 {
		value /= 1024
		unit++
	}
	return fmt.Sprintf("%.1f %s", value, units[unit])
}

func postJSON(ctx context.Context, client *http.Client, url string, body any, out any) error {
	payload, err := json.Marshal(body)
	if err != nil {
		return err
	}
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, url, bytes.NewReader(payload))
	if err != nil {
		return err
	}
	req.Header.Set("Content-Type", "application/json")
	resp, err := client.Do(req)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	var envelope HTTPEnvelope
	if err = json.NewDecoder(resp.Body).Decode(&envelope); err != nil {
		return err
	}
	if envelope.Code != 0 {
		if envelope.Message == "" {
			envelope.Message = "request failed"
		}
		return errors.New(envelope.Message)
	}
	if out != nil && len(envelope.Data) > 0 {
		return json.Unmarshal(envelope.Data, out)
	}
	return nil
}

func (s *AgentState) pullServerAssets(ctx context.Context, cfg Config) ([]ServerAssetEntry, error) {
	client := &http.Client{Timeout: 15 * time.Second}
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, fmt.Sprintf("%s/admin/client/assets/pull?deviceId=%d", cfg.HTTPBase, cfg.DeviceID), nil)
	if err != nil {
		return nil, err
	}
	resp, err := client.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	var envelope struct {
		Code    int    `json:"code"`
		Message string `json:"message"`
		Data    struct {
			DeviceID uint64             `json:"deviceId"`
			Assets   []ServerAssetEntry `json:"assets"`
		} `json:"data"`
	}
	if err = json.NewDecoder(resp.Body).Decode(&envelope); err != nil {
		return nil, err
	}
	if envelope.Code != 0 {
		if envelope.Message == "" {
			envelope.Message = "pull server assets failed"
		}
		return nil, errors.New(envelope.Message)
	}
	return envelope.Data.Assets, nil
}

func sendTCP(conn net.Conn, router string, data any) error {
	msg := TCPMessage{Router: router, Data: data}
	b, err := json.Marshal(msg)
	if err != nil {
		return err
	}
	if len(b) > 0xFFFF {
		return errors.New("tcp payload too large")
	}
	packet := make([]byte, 2+len(b))
	binary.BigEndian.PutUint16(packet[:2], uint16(len(b)))
	copy(packet[2:], b)
	_, err = conn.Write(packet)
	return err
}

func readTCPResponse(conn net.Conn, expected string) error {
	_ = conn.SetReadDeadline(time.Now().Add(20 * time.Second))
	packet, err := recvTCPPacket(conn)
	if err != nil {
		return err
	}
	var envelope TCPEnvelope
	if err := json.Unmarshal(packet, &envelope); err != nil {
		return err
	}
	if envelope.Router != expected {
		return fmt.Errorf("unexpected tcp response: %s", envelope.Router)
	}
	var res TCPResponse
	if err := json.Unmarshal(envelope.Data, &res); err != nil {
		return err
	}
	if res.Code != 0 && res.Code != 2000 {
		return errors.New(res.Message)
	}
	return nil
}

func recvTCPPacket(conn net.Conn) ([]byte, error) {
	header := make([]byte, 2)
	if _, err := io.ReadFull(conn, header); err != nil {
		return nil, err
	}
	length := binary.BigEndian.Uint16(header)
	if length == 0 {
		return nil, nil
	}
	body := make([]byte, length)
	if _, err := io.ReadFull(conn, body); err != nil {
		return nil, err
	}
	return body, nil
}

func (s *AgentState) saveConfig(serverHost, deviceName string) (Config, error) {
	serverHost = strings.TrimSpace(serverHost)
	deviceName = strings.TrimSpace(deviceName)
	if serverHost == "" || deviceName == "" {
		return s.getConfig(), errors.New("serverHost and deviceName are required")
	}

	s.mu.Lock()
	defer s.mu.Unlock()

	bindingChanged := s.config.ServerHost != serverHost || s.config.DeviceName != deviceName
	if bindingChanged {
		s.config.DeviceID = 0
		s.config.Token = ""
		s.config.TCPAddress = ""
		s.config.Hostname = ""
	}
	s.config.ServerHost = serverHost
	s.config.DeviceName = deviceName
	s.config.HTTPBase = normalizeHTTPBase(serverHost)

	if err := saveConfigFile(s.configPath, s.config); err != nil {
		return s.config, err
	}
	return s.config, nil
}

func (s *AgentState) loadConfig() error {
	cfg, err := loadConfigFile(s.configPath)
	if err != nil {
		if errors.Is(err, os.ErrNotExist) {
			return nil
		}
		return err
	}
	s.mu.Lock()
	s.config = cfg
	s.mu.Unlock()
	return nil
}

func (s *AgentState) getConfig() Config {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.config
}

func (s *AgentState) setConfig(cfg Config) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.config = cfg
}

func (s *AgentState) getStatus() AgentStatus {
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.status
}

func (s *AgentState) updateStatus(state string, deviceID uint64, tcpAddress, message string) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.status = AgentStatus{
		State:      state,
		DeviceID:   deviceID,
		TCPAddress: tcpAddress,
		Message:    message,
		UpdatedAt:  time.Now().UnixMilli(),
	}
}

func (s *AgentState) markStopped() {
	s.mu.Lock()
	defer s.mu.Unlock()
	if s.cancel != nil {
		s.cancel = nil
	}
	s.running = false
	if s.status.State != "stopped" {
		s.status.UpdatedAt = time.Now().UnixMilli()
	}
}

func resolveConfigPath(path string) (string, error) {
	if strings.TrimSpace(path) != "" {
		return path, nil
	}
	dir, err := os.UserConfigDir()
	if err != nil {
		return "", err
	}
	return filepath.Join(dir, "AuroraOpsClient", "agent-config.json"), nil
}

func loadConfigFile(path string) (Config, error) {
	content, err := os.ReadFile(path)
	if err != nil {
		return Config{}, err
	}
	var cfg Config
	if err = json.Unmarshal(content, &cfg); err != nil {
		return Config{}, err
	}
	if cfg.ServerHost != "" {
		cfg.HTTPBase = normalizeHTTPBase(cfg.ServerHost)
	}
	return cfg, nil
}

func saveConfigFile(path string, cfg Config) error {
	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		return err
	}
	content, err := json.MarshalIndent(cfg, "", "  ")
	if err != nil {
		return err
	}
	return os.WriteFile(path, content, 0o600)
}

func normalizeHTTPBase(host string) string {
	host = strings.TrimSpace(host)
	if strings.HasPrefix(host, "http://") || strings.HasPrefix(host, "https://") {
		return strings.TrimRight(host, "/")
	}
	return "http://" + strings.TrimRight(host, "/")
}

func detectIP() string {
	addrs, err := net.InterfaceAddrs()
	if err != nil {
		return ""
	}
	for _, addr := range addrs {
		if ipNet, ok := addr.(*net.IPNet); ok && !ipNet.IP.IsLoopback() && ipNet.IP.To4() != nil {
			return ipNet.IP.String()
		}
	}
	return ""
}

func sanitize(s string) string {
	return strings.ReplaceAll(strings.ToLower(strings.TrimSpace(s)), " ", "-")
}

func errString(err error) string {
	if err == nil {
		return ""
	}
	return err.Error()
}

func waitForSignal() {
	ch := make(chan os.Signal, 1)
	signal.Notify(ch, os.Interrupt, syscall.SIGTERM)
	<-ch
}

func writeJSON(w http.ResponseWriter, code int, payload any) {
	w.Header().Set("Content-Type", "application/json; charset=utf-8")
	w.WriteHeader(code)
	_ = json.NewEncoder(w).Encode(payload)
}
