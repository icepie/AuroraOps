package pay

import (
	"context"

	v1 "auroraops/api/api/pay/v1"
	"auroraops/internal/consts"
	"auroraops/internal/library/response"
	"auroraops/internal/model/input/payin"
	"auroraops/internal/service"

	"github.com/gogf/gf/v2/frame/g"
)

func (c *ControllerV1) NotifyWxPay(ctx context.Context, req *v1.NotifyWxPayReq) (res *v1.NotifyWxPayRes, err error) {
	if _, err = service.Pay().Notify(ctx, &payin.PayNotifyInp{PayType: consts.PayTypeWxPay}); err != nil {
		return
	}

	response.CustomJson(g.RequestFromCtx(ctx), `{"code": "SUCCESS","message": "收单成功"}`)
	return
}
