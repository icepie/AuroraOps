//go:build windows

package main

import (
	"context"
	"fmt"
	"os"
	"path/filepath"

	"golang.org/x/sys/windows/svc"
	"golang.org/x/sys/windows/svc/eventlog"
	"golang.org/x/sys/windows/svc/mgr"
)

const serviceName = "AuroraOpsAgent"

func handleServiceCommand(options RunOptions) error {
	switch options.ServiceCmd {
	case "install":
		return installWindowsService(options)
	case "uninstall":
		return uninstallWindowsService()
	case "run":
		return runWindowsService(options)
	default:
		return fmt.Errorf("unsupported service command: %s", options.ServiceCmd)
	}
}

func installWindowsService(options RunOptions) error {
	exePath, err := os.Executable()
	if err != nil {
		return err
	}

	m, err := mgr.Connect()
	if err != nil {
		return err
	}
	defer m.Disconnect()

	if service, err := m.OpenService(serviceName); err == nil {
		service.Close()
		return fmt.Errorf("service %s already exists", serviceName)
	}

	args := []string{
		"--service", "run",
		"--config", options.ConfigPath,
		"--port", fmt.Sprintf("%d", options.Port),
		"--headless",
	}
	if options.ServerHost != "" {
		args = append(args, "--server", options.ServerHost)
	}
	if options.DeviceName != "" {
		args = append(args, "--name", options.DeviceName)
	}

	service, err := m.CreateService(serviceName, exePath, mgr.Config{
		DisplayName: serviceName,
		StartType:   mgr.StartAutomatic,
	}, args...)
	if err != nil {
		return err
	}
	defer service.Close()

	_ = eventlog.InstallAsEventCreate(serviceName, eventlog.Error|eventlog.Warning|eventlog.Info)
	return nil
}

func uninstallWindowsService() error {
	m, err := mgr.Connect()
	if err != nil {
		return err
	}
	defer m.Disconnect()

	service, err := m.OpenService(serviceName)
	if err != nil {
		return err
	}
	defer service.Close()

	if err = service.Delete(); err != nil {
		return err
	}
	_ = eventlog.Remove(serviceName)
	return nil
}

func runWindowsService(options RunOptions) error {
	isWindowsService, err := svc.IsWindowsService()
	if err != nil {
		return err
	}
	if !isWindowsService {
		return runAgent(context.Background(), options)
	}
	return svc.Run(serviceName, &windowsService{options: options})
}

type windowsService struct {
	options RunOptions
}

func (m *windowsService) Execute(args []string, r <-chan svc.ChangeRequest, s chan<- svc.Status) (bool, uint32) {
	const cmdsAccepted = svc.AcceptStop | svc.AcceptShutdown
	s <- svc.Status{State: svc.StartPending}

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	errCh := make(chan error, 1)
	go func() {
		errCh <- runAgent(ctx, m.options)
	}()

	s <- svc.Status{State: svc.Running, Accepts: cmdsAccepted}

	for {
		select {
		case c := <-r:
			switch c.Cmd {
			case svc.Interrogate:
				s <- c.CurrentStatus
			case svc.Stop, svc.Shutdown:
				s <- svc.Status{State: svc.StopPending}
				cancel()
				<-errCh
				return false, 0
			default:
			}
		case err := <-errCh:
			if err != nil {
				return false, 1
			}
			return false, 0
		}
	}
}

func init() {
	_ = os.MkdirAll(filepath.Dir(filepath.Clean(os.TempDir())), 0o755)
}
