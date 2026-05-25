// =================================================================================
// Code generated and maintained by GoFrame CLI tool. DO NOT EDIT.
// =================================================================================

package member

import (
	"context"

	"auroraops/api/api/member/v1"
)

type IMemberV1 interface {
	GetIdByCode(ctx context.Context, req *v1.GetIdByCodeReq) (res *v1.GetIdByCodeRes, err error)
}
