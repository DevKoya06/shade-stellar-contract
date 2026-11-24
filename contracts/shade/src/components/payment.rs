use crate::errors::ContractError;
use crate::types::{PaymentPayload, PaymentRoute};
use soroban_sdk::{panic_with_error, Env};

const MAX_SLIPPAGE_BPS: u32 = 10_000;

// Payment routing note: the payment flow in this module only validates the payload.
// Token transfers for payments are settled directly to their destinations (the
// merchant and the platform fee account) and are never parked on the hub, so
// no platform revenue balance accumulates here. Because nothing builds up on the
// hub from payments, no admin revenue withdrawal function is required from this
// module. Residual hub balances from other flows (rounding dust, mistaken direct
// transfers) are addressed in the module that owns those flows. See
// docs/concepts/fees.md for the full platform fee routing.

pub fn validate_payment_payload(env: &Env, payload: &PaymentPayload) {
    match &payload.route {
        PaymentRoute::Direct => {
            if payload.input_token != payload.settlement_token {
                panic_with_error!(env, ContractError::InvalidSwapPath);
            }

            if payload.max_slippage_bps.is_some() {
                panic_with_error!(env, ContractError::InvalidSlippage);
            }
        }
        PaymentRoute::Swap(route) => {
            if route.path.len() < 2 {
                panic_with_error!(env, ContractError::InvalidSwapPath);
            }

            let first_hop = route
                .path
                .get(0)
                .unwrap_or_else(|| panic_with_error!(env, ContractError::InvalidSwapPath));
            let last_hop = route
                .path
                .get(route.path.len() - 1)
                .unwrap_or_else(|| panic_with_error!(env, ContractError::InvalidSwapPath));

            if first_hop != payload.input_token
                || last_hop != payload.settlement_token
                || payload.input_token == payload.settlement_token
            {
                panic_with_error!(env, ContractError::InvalidSwapPath);
            }

            let slippage_bps = payload
                .max_slippage_bps
                .unwrap_or_else(|| panic_with_error!(env, ContractError::InvalidSlippage));

            if slippage_bps > MAX_SLIPPAGE_BPS {
                panic_with_error!(env, ContractError::InvalidSlippage);
            }
        }
    }
}
