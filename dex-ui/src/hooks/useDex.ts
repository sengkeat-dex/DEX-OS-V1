import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  loadWasm,
  WasmAMM,
  WasmOrder,
  WasmOrderBook,
  WasmTrade
} from "../wasmBridge";

type DexStatus = "idle" | "loading" | "ready" | "error";

interface DexState {
  status: DexStatus;
  lastMessage: string;
  bestBid?: number;
  bestAsk?: number;
  lastTrades: WasmTrade[];
  error?: string;
}

const INITIAL_STATE: DexState = {
  status: "idle",
  lastMessage: "Awaiting WASM initialization",
  lastTrades: []
};

/**
 * React hook that wires the wasm-bindgen interface into the UI.
 * Exposes a demo action so contributors can verify everything locally.
 */
export function useDex() {
  const [state, setState] = useState<DexState>(INITIAL_STATE);
  const orderBookRef = useRef<WasmOrderBook | null>(null);
  const ammRef = useRef<WasmAMM | null>(null);

  const initialize = useCallback(async () => {
    setState((prev) => ({
      ...prev,
      status: "loading",
      lastMessage: "Loading wasm bundle..."
    }));

    try {
      const wasm = await loadWasm();
      orderBookRef.current = new wasm.WasmOrderBook();
      ammRef.current = new wasm.WasmAMM(30);
      setState({
        status: "ready",
        lastMessage: "WASM module ready. Try the demo action.",
        bestBid: undefined,
        bestAsk: undefined,
        lastTrades: []
      });
    } catch (error) {
      setState({
        status: "error",
        lastMessage: "Failed to load WASM module.",
        lastTrades: [],
        error: error instanceof Error ? error.message : String(error)
      });
    }
  }, []);

  useEffect(() => {
    void initialize();
  }, [initialize]);

  const runDemo = useCallback(async () => {
    const orderBook = orderBookRef.current;
    const amm = ammRef.current;

    if (!orderBook || !amm) {
      return setState((prev) => ({
        ...prev,
        error: "WASM bindings not initialized yet.",
        lastMessage: "Initialization still in progress..."
      }));
    }

    try {
      // Seed AMM reserves so quotes make sense
      amm.add_liquidity("BTC", 1_000_000, "USD", 50_000_000);

      const sellOrder: WasmOrder = {
        id: Number(Date.now()),
        trader_id: "demo-seller",
        pair: { base: "BTC", quote: "USD" },
        side: "Sell",
        order_type: "Limit",
        price: 50_000,
        quantity: 50_000,
        timestamp: Math.floor(Date.now() / 1000)
      };

      const buyOrder: WasmOrder = {
        id: Number(Date.now() + 1),
        trader_id: "demo-buyer",
        pair: { base: "BTC", quote: "USD" },
        side: "Buy",
        order_type: "Limit",
        price: 50_000,
        quantity: 25_000,
        timestamp: Math.floor(Date.now() / 1000)
      };

      orderBook.add_order(sellOrder);
      const trades = orderBook.add_order(buyOrder);
      const bestBid = orderBook.best_bid();
      const bestAsk = orderBook.best_ask();

      setState({
        status: "ready",
        lastMessage: `Executed ${trades.length} trade(s).`,
        lastTrades: trades,
        bestBid: typeof bestBid === "number" ? bestBid : undefined,
        bestAsk: typeof bestAsk === "number" ? bestAsk : undefined
      });
    } catch (error) {
      setState((prev) => ({
        ...prev,
        status: "error",
        error: error instanceof Error ? error.message : String(error),
        lastMessage: "Demo interaction failed."
      }));
    }
  }, []);

  return useMemo(
    () => ({
      ...state,
      initialize,
      runDemo
    }),
    [state, initialize, runDemo]
  );
}
