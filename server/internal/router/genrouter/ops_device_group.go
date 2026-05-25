package genrouter

import "auroraops/internal/controller/admin/sys"

func init() {
	LoginRequiredRouter = append(LoginRequiredRouter, sys.OpsDeviceGroup)
}
