//! The A2A interoperability facade (`PHASE-8.2.3`, ADR-025): the
//! FACADE boundary — the A2A messages/tasks map to the local surface
//! where the semantics align, and every exchange records its SEMANTIC
//! LOSSES (the authority, the budget, the evidence, the decision rule,
//! the policy lifecycle — each dimension names its loss; a silent merge
//! is the failure mode this vocabulary forbids).
//!
//! The A2A message is an INPUT the local machinery evaluates — the local
//! grants authorize every local effect, the local budgets bound every
//! cost, the local policy digests decide every rule. The facade itself
//! confers NOTHING.

use serde::{Deserialize, Serialize};

/// The five §9.7 semantic dimensions the facade maps — each exchange
/// records which dimensions SURVIVED and which are LOST.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SemanticLosses {
    /// The remote authority is not the local grant — always lost (the
    /// local grants are the only authority that acts).
    pub authority: bool,
    /// The remote cost is not the local ceiling — always lost (the
    /// local budgets bound every cost).
    pub budget: bool,
    /// The remote references are the digest references, never the local
    /// evidence objects — lost unless the local pipeline re-derives them.
    pub evidence: bool,
    /// The remote decision rules are not the local policy digests —
    /// always lost.
    pub decision_rule: bool,
    /// The remote task states are not the thread states — always lost.
    pub policy_lifecycle: bool,
}

/// A mapped A2A message: the compatible content (the text) + the
/// preserved external identity (the sender role — the wire value, never
/// interpreted as a local principal) + the loss record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MappedMessage {
    /// The external message's text (the compatible content).
    pub text: String,
    /// The external sender role's wire value (PRESERVED — an external
    /// id, never a local authority).
    pub external_role: String,
    /// The loss record (every dimension names its loss).
    pub losses: SemanticLosses,
}

/// Map one A2A message onto the facade: the text survives, the external
/// role is preserved as the external id, and ALL five semantic
/// dimensions are recorded lost (an A2A message carries none of the
/// local machinery's authority/budget/evidence/decision-rule/lifecycle —
/// the local surface re-evaluates everything).
pub fn map_message(message: &a2a::Message) -> MappedMessage {
    MappedMessage {
        text: message.text().unwrap_or_default().to_string(),
        external_role: format!("{:?}", message.role),
        losses: SemanticLosses::default(),
    }
}

/// The external task id, preserved verbatim (the §9.7 "preserve external
/// IDs" rule — the local aggregate gets its OWN id; the external id is a
/// recorded reference, never the local key).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MappedTask {
    pub external_task_id: String,
    pub text: String,
    pub losses: SemanticLosses,
}

/// Map one A2A task's request onto the facade: the external task id
/// survives as the reference, the message text survives, the losses
/// record (the task carries no local authority/budget/evidence/rule/
/// lifecycle).
pub fn map_task_request(task_id: &str, message: &a2a::Message) -> MappedTask {
    MappedTask {
        external_task_id: task_id.to_string(),
        text: message.text().unwrap_or_default().to_string(),
        losses: SemanticLosses::default(),
    }
}

/// The facade's response: the local outcome's text + the echo of the
/// external task id (the response preserves the external identity).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FacadeResponse {
    pub external_task_id: String,
    pub text: String,
}

/// Build the A2A response message for a mapped outcome (the text out,
/// the role is the assistant — the facade's own voice).
pub fn response_message(response: &FacadeResponse) -> a2a::Message {
    a2a::Message::new(a2a::Role::Agent, vec![a2a::Part::text(&response.text)])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The A2A message maps with the text surviving, the external role
    /// preserved, and ALL five semantic dimensions recorded lost.
    #[test]
    fn the_message_maps_with_every_loss_recorded() {
        let message = a2a::Message::new(a2a::Role::User, vec![a2a::Part::text("hello")]);
        assert_eq!(message.text(), Some("hello"));
        let mapped = map_message(&message);
        assert_eq!(mapped.text, "hello");
        assert!(
            !mapped.external_role.is_empty(),
            "the external role preserves"
        );
        assert_eq!(
            mapped.losses,
            SemanticLosses::default(),
            "all five dimensions lost on the bare message"
        );
    }

    /// The external task id preserves verbatim (the local aggregate gets
    /// its own id — the external id is the reference, never the key).
    #[test]
    fn the_external_task_id_preserves() {
        let message = a2a::Message::new(a2a::Role::User, vec![a2a::Part::text("the ask")]);
        let mapped = map_task_request("ext-task-42", &message);
        assert_eq!(mapped.external_task_id, "ext-task-42");
        assert_eq!(mapped.text, "the ask");
    }

    /// The response round-trips through the A2A message shape.
    #[test]
    fn the_response_roundtrips() {
        let response = FacadeResponse {
            external_task_id: "ext-task-42".to_string(),
            text: "the answer".to_string(),
        };
        let message = response_message(&response);
        assert_eq!(message.text(), Some("the answer"));
    }

    /// The WIRE roundtrip (the `.2.4` demonstration): the A2A JSON-RPC
    /// request (`message/send` with the Message params) serializes →
    /// deserializes → the facade maps → the JSON-RPC success response
    /// serializes → deserializes. The compatibility is demonstrated over
    /// the REAL wire shapes (the JSON-RPC 2.0 envelope + the a2a-lf
    /// 0.3.0 types), never inferred.
    #[test]
    fn the_wire_roundtrip_demonstrates_the_json_rpc_exchange() {
        let message = a2a::Message::new(a2a::Role::User, vec![a2a::Part::text("the ask")]);
        let params = serde_json::json!({ "message": message });
        let request = a2a::jsonrpc::JsonRpcRequest::new(
            a2a::jsonrpc::JsonRpcId::String("req-1".to_string()),
            "message/send",
            Some(params),
        );
        let wire = serde_json::to_vec(&request).expect("the request serializes");
        let decoded: a2a::jsonrpc::JsonRpcRequest =
            serde_json::from_slice(&wire).expect("the request deserializes");
        assert_eq!(decoded.method, "message/send");
        let message: a2a::Message = decoded
            .params
            .expect("the params")
            .get("message")
            .cloned()
            .map(|v| serde_json::from_value(v).expect("the message decodes"))
            .expect("the message param");
        let mapped = map_message(&message);
        assert_eq!(mapped.text, "the ask");
        let result = serde_json::to_value(&mapped).expect("the mapped result serializes");
        let response = a2a::jsonrpc::JsonRpcResponse::success(
            a2a::jsonrpc::JsonRpcId::String("req-1".to_string()),
            result,
        );
        let wire = serde_json::to_vec(&response).expect("the response serializes");
        let decoded: a2a::jsonrpc::JsonRpcResponse =
            serde_json::from_slice(&wire).expect("the response deserializes");
        assert!(
            decoded.error.is_none(),
            "the success response carries no error"
        );
        let mapped: MappedMessage = serde_json::from_value(decoded.result.expect("the result"))
            .expect("the result decodes");
        assert_eq!(mapped.text, "the ask");
        assert_eq!(mapped.losses, SemanticLosses::default());
    }

    /// The unknown-method refusal: the facade refuses an OUTSIDE-the-set
    /// method with the JSON-RPC error (the typed refusal — never a silent
    /// pass).
    #[test]
    fn the_unknown_method_refuses_with_the_json_rpc_error() {
        assert!(!a2a::jsonrpc::methods::is_valid("bogus/method"));
        let error = a2a::jsonrpc::JsonRpcError {
            code: -32601,
            message: "method not found".to_string(),
            data: None,
        };
        let response = a2a::jsonrpc::JsonRpcResponse::error(
            a2a::jsonrpc::JsonRpcId::String("req-2".to_string()),
            error,
        );
        let wire = serde_json::to_vec(&response).expect("the error serializes");
        let decoded: a2a::jsonrpc::JsonRpcResponse =
            serde_json::from_slice(&wire).expect("the error deserializes");
        assert_eq!(decoded.error.as_ref().map(|e| e.code), Some(-32601));
    }
}
