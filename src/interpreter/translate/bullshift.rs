use anyhow::Result;

use super::mcp_helper::{insert_opt, insert_str, mcp_call};
use crate::interpreter::intent::{Intent, Translation};
use crate::security::PermissionLevel;

pub(crate) fn translate_bullshift(intent: &Intent) -> Result<Translation> {
    match intent {
        Intent::BullShiftPortfolio { action, period } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "period", period);
            Ok(mcp_call(
                "bullshift_portfolio",
                a,
                format!("BullShift portfolio: {}", action),
                PermissionLevel::Safe,
                format!("Views portfolio {} via BullShift MCP bridge", action),
            ))
        }

        Intent::BullShiftOrders {
            action,
            symbol,
            side,
        } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "symbol", symbol);
            insert_opt(&mut a, "side", side);
            Ok(mcp_call(
                "bullshift_orders",
                a,
                format!("BullShift order: {}", action),
                match action.as_str() {
                    "list" | "status" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                format!(
                    "{} orders via BullShift MCP bridge",
                    match action.as_str() {
                        "place" => "Places",
                        "cancel" => "Cancels",
                        "list" => "Lists",
                        "status" => "Checks",
                        _ => "Manages",
                    }
                ),
            ))
        }

        Intent::BullShiftMarket { action, symbol } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "symbol", symbol);
            Ok(mcp_call(
                "bullshift_market",
                a,
                format!(
                    "BullShift market: {}{}",
                    action,
                    symbol
                        .as_ref()
                        .map_or(String::new(), |s| format!(" for {}", s))
                ),
                PermissionLevel::Safe,
                "Queries market data via BullShift MCP bridge".to_string(),
            ))
        }

        Intent::BullShiftAlerts { action, symbol } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "symbol", symbol);
            Ok(mcp_call(
                "bullshift_alerts",
                a,
                format!("BullShift alert: {}", action),
                match action.as_str() {
                    "list" | "triggered" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                format!(
                    "{} price alerts via BullShift MCP bridge",
                    match action.as_str() {
                        "set" => "Sets",
                        "remove" => "Removes",
                        "list" => "Lists",
                        "triggered" => "Lists triggered",
                        _ => "Manages",
                    }
                ),
            ))
        }

        Intent::BullShiftStrategy { action, name } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "name", name);
            Ok(mcp_call(
                "bullshift_strategy",
                a,
                format!("BullShift strategy: {}", action),
                match action.as_str() {
                    "list" | "status" => PermissionLevel::Safe,
                    _ => PermissionLevel::SystemWrite,
                },
                format!(
                    "{} trading strategy via BullShift MCP bridge",
                    match action.as_str() {
                        "list" => "Lists",
                        "start" => "Starts",
                        "stop" => "Stops",
                        "backtest" => "Backtests",
                        "status" => "Checks status of",
                        _ => "Manages",
                    }
                ),
            ))
        }

        Intent::BullShiftAccounts { action, broker } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "broker", broker);
            Ok(mcp_call(
                "bullshift_accounts",
                a,
                format!(
                    "BullShift accounts: {}{}",
                    action,
                    broker
                        .as_ref()
                        .map_or(String::new(), |b| format!(" '{}'", b))
                ),
                PermissionLevel::Safe,
                "Views broker accounts via BullShift".to_string(),
            ))
        }

        Intent::BullShiftHistory { action, period } => {
            let mut a = serde_json::Map::new();
            insert_str(&mut a, "action", action);
            insert_opt(&mut a, "period", period);
            Ok(mcp_call(
                "bullshift_history",
                a,
                format!(
                    "BullShift history: {}{}",
                    action,
                    period
                        .as_ref()
                        .map_or(String::new(), |p| format!(" ({})", p))
                ),
                PermissionLevel::Safe,
                "Views trade history via BullShift".to_string(),
            ))
        }

        _ => unreachable!("translate_bullshift called with non-bullshift intent"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_portfolio_with_period() {
        let intent = Intent::BullShiftPortfolio {
            action: "summary".to_string(),
            period: Some("1m".to_string()),
        };
        let result = translate_bullshift(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "bullshift_portfolio");
        assert_eq!(mcp.arguments["action"], "summary");
        assert_eq!(mcp.arguments["period"], "1m");
        assert_eq!(result.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_portfolio_without_period() {
        let intent = Intent::BullShiftPortfolio {
            action: "summary".to_string(),
            period: None,
        };
        let result = translate_bullshift(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "bullshift_portfolio");
        assert!(mcp.arguments.get("period").is_none());
    }

    #[test]
    fn test_orders_list() {
        let intent = Intent::BullShiftOrders {
            action: "list".to_string(),
            symbol: None,
            side: None,
        };
        let result = translate_bullshift(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "bullshift_orders");
        assert_eq!(mcp.arguments["action"], "list");
        assert_eq!(result.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_orders_place() {
        let intent = Intent::BullShiftOrders {
            action: "place".to_string(),
            symbol: Some("AAPL".to_string()),
            side: Some("buy".to_string()),
        };
        let result = translate_bullshift(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "bullshift_orders");
        assert_eq!(mcp.arguments["action"], "place");
        assert_eq!(mcp.arguments["symbol"], "AAPL");
        assert_eq!(mcp.arguments["side"], "buy");
        assert_eq!(result.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_orders_cancel() {
        let intent = Intent::BullShiftOrders {
            action: "cancel".to_string(),
            symbol: Some("TSLA".to_string()),
            side: None,
        };
        let result = translate_bullshift(&intent).unwrap();
        assert_eq!(result.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_orders_status() {
        let intent = Intent::BullShiftOrders {
            action: "status".to_string(),
            symbol: None,
            side: None,
        };
        let result = translate_bullshift(&intent).unwrap();
        assert_eq!(result.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_market_with_symbol() {
        let intent = Intent::BullShiftMarket {
            action: "quote".to_string(),
            symbol: Some("GOOG".to_string()),
        };
        let result = translate_bullshift(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "bullshift_market");
        assert_eq!(mcp.arguments["action"], "quote");
        assert_eq!(mcp.arguments["symbol"], "GOOG");
        assert_eq!(result.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_market_without_symbol() {
        let intent = Intent::BullShiftMarket {
            action: "overview".to_string(),
            symbol: None,
        };
        let result = translate_bullshift(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert!(mcp.arguments.get("symbol").is_none());
    }

    #[test]
    fn test_alerts_list() {
        let intent = Intent::BullShiftAlerts {
            action: "list".to_string(),
            symbol: None,
        };
        let result = translate_bullshift(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "bullshift_alerts");
        assert_eq!(result.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_alerts_set() {
        let intent = Intent::BullShiftAlerts {
            action: "set".to_string(),
            symbol: Some("MSFT".to_string()),
        };
        let result = translate_bullshift(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.arguments["action"], "set");
        assert_eq!(mcp.arguments["symbol"], "MSFT");
        assert_eq!(result.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_alerts_triggered() {
        let intent = Intent::BullShiftAlerts {
            action: "triggered".to_string(),
            symbol: None,
        };
        let result = translate_bullshift(&intent).unwrap();
        assert_eq!(result.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_alerts_remove() {
        let intent = Intent::BullShiftAlerts {
            action: "remove".to_string(),
            symbol: Some("AMZN".to_string()),
        };
        let result = translate_bullshift(&intent).unwrap();
        assert_eq!(result.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_strategy_list() {
        let intent = Intent::BullShiftStrategy {
            action: "list".to_string(),
            name: None,
        };
        let result = translate_bullshift(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "bullshift_strategy");
        assert_eq!(result.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_strategy_start() {
        let intent = Intent::BullShiftStrategy {
            action: "start".to_string(),
            name: Some("momentum".to_string()),
        };
        let result = translate_bullshift(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.arguments["action"], "start");
        assert_eq!(mcp.arguments["name"], "momentum");
        assert_eq!(result.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_strategy_status() {
        let intent = Intent::BullShiftStrategy {
            action: "status".to_string(),
            name: Some("arb".to_string()),
        };
        let result = translate_bullshift(&intent).unwrap();
        assert_eq!(result.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_strategy_backtest() {
        let intent = Intent::BullShiftStrategy {
            action: "backtest".to_string(),
            name: Some("mean_revert".to_string()),
        };
        let result = translate_bullshift(&intent).unwrap();
        assert_eq!(result.permission, PermissionLevel::SystemWrite);
    }

    #[test]
    fn test_accounts_with_broker() {
        let intent = Intent::BullShiftAccounts {
            action: "list".to_string(),
            broker: Some("alpaca".to_string()),
        };
        let result = translate_bullshift(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "bullshift_accounts");
        assert_eq!(mcp.arguments["action"], "list");
        assert_eq!(mcp.arguments["broker"], "alpaca");
        assert_eq!(result.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_accounts_without_broker() {
        let intent = Intent::BullShiftAccounts {
            action: "list".to_string(),
            broker: None,
        };
        let result = translate_bullshift(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert!(mcp.arguments.get("broker").is_none());
    }

    #[test]
    fn test_history_with_period() {
        let intent = Intent::BullShiftHistory {
            action: "trades".to_string(),
            period: Some("7d".to_string()),
        };
        let result = translate_bullshift(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.tool_name, "bullshift_history");
        assert_eq!(mcp.arguments["action"], "trades");
        assert_eq!(mcp.arguments["period"], "7d");
        assert_eq!(result.permission, PermissionLevel::Safe);
    }

    #[test]
    fn test_history_without_period() {
        let intent = Intent::BullShiftHistory {
            action: "pnl".to_string(),
            period: None,
        };
        let result = translate_bullshift(&intent).unwrap();
        let mcp = result.mcp.as_ref().unwrap();
        assert_eq!(mcp.arguments["action"], "pnl");
        assert!(mcp.arguments.get("period").is_none());
    }
}
