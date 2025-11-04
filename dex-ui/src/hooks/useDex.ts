import { useCallback, useEffect, useMemo, useState } from "react";
import { DexApiClient } from "../api/client";
import { ApiDepthSnapshot, ApiTrade, OrderSide, OrderType } from "../api/types";
import { DEFAULT_API_BASE_URL } from "../config";

const SESSION_STORAGE_KEY = "dex-ui/session";

export type DexStatus = "idle" | "connecting" | "ready" | "error";

export interface OrderFormValues {
  baseToken: string;
  quoteToken: string;
  side: OrderSide;
  orderType: OrderType;
  price?: number;
  quantity: number;
}

interface MarketState {
  bestBid?: number;
  bestAsk?: number;
  lastUpdated?: string;
  isLoading: boolean;
}

interface WalletState {
  address?: string;
  traderId?: string;
  token?: string;
}

type NotificationType = "success" | "error" | "info";

interface Notification {
  id: number;
  type: NotificationType;
  message: string;
}

type DepthConnection = "idle" | "connecting" | "live" | "error";

interface DepthState {
  snapshot: ApiDepthSnapshot | null;
  connection: DepthConnection;
  lastError?: string;
  isLoading: boolean;
}

interface UseDexReturn {
  status: DexStatus;
  apiBaseUrl: string;
  setApiBaseUrl: (value: string) => void;
  wallet: WalletState;
  setTraderId: (value: string) => void;
  setAuthToken: (value: string) => void;
  connectWallet: () => Promise<void>;
  market: MarketState;
  refreshMarketData: () => Promise<void>;
  depth: DepthState;
  refreshDepth: () => Promise<void>;
  trades: ApiTrade[];
  loadTrades: (traderId?: string) => Promise<ApiTrade[] | undefined>;
  isRefreshingMarket: boolean;
  isFetchingTrades: boolean;
  isSubmittingOrder: boolean;
  isIssuingToken: boolean;
  issueTokenWithSecret: (secret: string, ttlSeconds?: number) => Promise<void>;
  issueTokenWithWallet: (ttlSeconds?: number) => Promise<void>;
  submitOrder: (values: OrderFormValues) => Promise<void>;
  lastOrderId: number | null;
  notification: Notification | null;
  dismissNotification: () => void;
}

export function useDex(): UseDexReturn {
  const depthLevels = 10;
  const [status, setStatus] = useState<DexStatus>("idle");
  const [apiBaseUrl, setApiBaseUrlState] = useState(DEFAULT_API_BASE_URL);
  const [walletAddress, setWalletAddress] = useState<string | undefined>();
  const [traderId, setTraderIdState] = useState<string | undefined>();
  const [token, setToken] = useState<string | undefined>();
  const [market, setMarket] = useState<MarketState>({ isLoading: false });
  const [trades, setTrades] = useState<ApiTrade[]>([]);
  const [isRefreshingMarket, setIsRefreshingMarket] = useState(false);
  const [isFetchingTrades, setIsFetchingTrades] = useState(false);
  const [isSubmittingOrder, setIsSubmittingOrder] = useState(false);
  const [isIssuingToken, setIsIssuingToken] = useState(false);
  const [lastOrderId, setLastOrderId] = useState<number | null>(null);
  const [notification, setNotification] = useState<Notification | null>(null);
  const [depth, setDepth] = useState<DepthState>({
    snapshot: null,
    connection: "idle",
    isLoading: false,
  });

  const apiClient = useMemo(
    () => new DexApiClient({ baseUrl: apiBaseUrl, token }),
    [apiBaseUrl, token]
  );

  const notify = useCallback((type: NotificationType, message: string) => {
    setNotification({
      id: Date.now(),
      type,
      message
    });
  }, []);

  const dismissNotification = useCallback(() => {
    setNotification(null);
  }, []);

  const refreshDepth = useCallback(async () => {
    setDepth((prev) => ({ ...prev, isLoading: true }));
    try {
      const snapshot = await apiClient.fetchDepth(depthLevels);
      setDepth((prev) => ({
        ...prev,
        snapshot,
        isLoading: false,
        lastError: undefined,
      }));
    } catch (error) {
      const message = error instanceof Error ? error.message : "Failed to load depth.";
      setDepth((prev) => ({
        ...prev,
        isLoading: false,
        lastError: message,
        connection: prev.connection === "live" ? "error" : prev.connection,
      }));
    }
  }, [apiClient, depthLevels]);

  useEffect(() => {
    if (!notification) {
      return;
    }
    const timer = window.setTimeout(() => setNotification(null), 6000);
    return () => window.clearTimeout(timer);
  }, [notification]);

  useEffect(() => {
    if (typeof window === "undefined") {
      return;
    }
    try {
      const raw = window.localStorage.getItem(SESSION_STORAGE_KEY);
      if (!raw) {
        return;
      }
      const snapshot = JSON.parse(raw) as {
        apiBaseUrl?: string;
        traderId?: string;
        token?: string;
      };
      if (snapshot.apiBaseUrl) {
        setApiBaseUrlState(snapshot.apiBaseUrl);
      }
      if (snapshot.traderId) {
        setTraderIdState(snapshot.traderId);
      }
      if (snapshot.token) {
        setToken(snapshot.token);
      }
    } catch {
      // ignore corrupted session payloads
    }
  }, []);

  useEffect(() => {
    if (typeof window === "undefined") {
      return;
    }
    const payload = JSON.stringify({
      apiBaseUrl,
      traderId,
      token
    });
    window.localStorage.setItem(SESSION_STORAGE_KEY, payload);
  }, [apiBaseUrl, traderId, token]);

  useEffect(() => {
    void refreshDepth();
  }, [refreshDepth]);

  const refreshMarketData = useCallback(async () => {
    setIsRefreshingMarket(true);
    setMarket((prev) => ({ ...prev, isLoading: true }));
    setStatus((prev) => (prev === "ready" ? prev : "connecting"));
    try {
      const prices = await apiClient.fetchPrices();
      setMarket({
        bestBid: prices.best_bid ?? undefined,
        bestAsk: prices.best_ask ?? undefined,
        lastUpdated: new Date().toISOString(),
        isLoading: false
      });
      setStatus("ready");
    } catch (error) {
      setStatus("error");
      notify("error", error instanceof Error ? error.message : "Failed to load prices");
      setMarket((prev) => ({ ...prev, isLoading: false }));
      throw error;
    } finally {
      setIsRefreshingMarket(false);
    }
  }, [apiClient, notify]);

  useEffect(() => {
    void refreshMarketData();
    const interval = window.setInterval(() => {
      void refreshMarketData();
    }, 15_000);
    return () => window.clearInterval(interval);
  }, [refreshMarketData]);

  useEffect(() => {
    let cancelled = false;
    let reconnectTimer: number | undefined;
    let socket: WebSocket | null = null;

    const connect = () => {
      if (cancelled) {
        return;
      }
      const url = buildDepthWsUrl(apiBaseUrl, depthLevels);
      if (!url) {
        setDepth((prev) => ({
          ...prev,
          connection: "error",
          lastError: "Invalid API base URL for depth stream.",
        }));
        return;
      }
      setDepth((prev) => ({ ...prev, connection: "connecting" }));
      socket = new WebSocket(url);

      socket.onopen = () => {
        if (cancelled) {
          return;
        }
        setDepth((prev) => ({ ...prev, connection: "live", lastError: undefined }));
      };

      socket.onmessage = (event) => {
        if (cancelled) {
          return;
        }
        try {
          const payload = JSON.parse(event.data) as ApiDepthSnapshot;
          setDepth((prev) => ({
            ...prev,
            snapshot: payload,
            connection: "live",
            lastError: undefined,
            isLoading: false,
          }));
        } catch (error) {
          setDepth((prev) => ({
            ...prev,
            lastError: "Failed to parse depth snapshot.",
          }));
        }
      };

      socket.onerror = () => {
        if (cancelled) {
          return;
        }
        setDepth((prev) => ({
          ...prev,
          connection: "error",
          lastError: "Depth stream error.",
        }));
      };

      socket.onclose = () => {
        if (cancelled) {
          return;
        }
        setDepth((prev) => ({ ...prev, connection: "error" }));
        reconnectTimer = window.setTimeout(connect, 3_000);
      };
    };

    connect();

    return () => {
      cancelled = true;
      if (reconnectTimer) {
        window.clearTimeout(reconnectTimer);
      }
      if (socket && (socket.readyState === WebSocket.OPEN || socket.readyState === WebSocket.CONNECTING)) {
        socket.close(1000, "cleanup");
      }
    };
  }, [apiBaseUrl, depthLevels]);

  const fetchTrades = useCallback(
    async (maybeTraderId?: string, silent = false) => {
      const effectiveTraderId = (maybeTraderId || traderId || "").trim();
      if (!effectiveTraderId || !token) {
        if (!silent) {
          notify("info", "Provide a trader ID and JWT token to load trades.");
        }
        return undefined;
      }

      setIsFetchingTrades(true);
      try {
        const response = await apiClient.fetchTradesForTrader(effectiveTraderId);
        setTrades(response.trades || []);
        if (!silent) {
          notify("success", `Loaded ${response.trades.length} trade(s).`);
        }
        return response.trades;
      } catch (error) {
        if (!silent) {
          notify("error", error instanceof Error ? error.message : "Failed to load trades");
        }
        throw error;
      } finally {
        setIsFetchingTrades(false);
      }
    },
    [apiClient, notify, token, traderId]
  );

  const loadTrades = useCallback(
    async (maybeTraderId?: string) => fetchTrades(maybeTraderId, false),
    [fetchTrades]
  );

  useEffect(() => {
    if (!traderId || !token) {
      setTrades([]);
      return;
    }
    void fetchTrades(traderId, true);
  }, [fetchTrades, token, traderId]);

  const setApiBaseUrl = useCallback((value: string) => {
    const sanitized = value.trim().replace(/\/+$/, "");
    setApiBaseUrlState(sanitized || DEFAULT_API_BASE_URL);
  }, []);

  const setTraderId = useCallback((value: string) => {
    const normalized = value.trim();
    setTraderIdState(normalized || undefined);
  }, []);

  const setAuthToken = useCallback((value: string) => {
    const normalized = value.trim();
    setToken(normalized || undefined);
  }, []);

  const connectWallet = useCallback(async () => {
    if (typeof window === "undefined" || !window.ethereum) {
      notify("error", "No Ethereum wallet detected. Install MetaMask or connect manually.");
      return;
    }
    try {
      const accounts = (await window.ethereum.request({
        method: "eth_requestAccounts"
      })) as string[];
      if (!accounts || accounts.length === 0) {
        throw new Error("Wallet did not return any accounts.");
      }
      const primary = accounts[0];
      setWalletAddress(primary);
      setTraderIdState((prev) => prev || primary);
      notify("success", `Wallet connected: ${shorten(primary)}`);
    } catch (error) {
      notify("error", error instanceof Error ? error.message : "Failed to connect wallet.");
    }
  }, [notify]);

  const issueTokenWithSecret = useCallback(
    async (secretValue: string, ttlSeconds?: number) => {
      const trader = (traderId || "").trim();
      if (!trader) {
        notify("error", "Set a trader ID before requesting a token.");
        return;
      }
      if (!secretValue.trim()) {
        notify("error", "Shared secret is required.");
        return;
      }
      setIsIssuingToken(true);
      try {
        const response = await apiClient.requestSharedToken({
          trader_id: trader,
          secret: secretValue.trim(),
          ttl_seconds: ttlSeconds
        });
        setToken(response.token);
        notify("success", "Issued JWT via shared secret.");
      } catch (error) {
        notify("error", error instanceof Error ? error.message : "Failed to issue token.");
      } finally {
        setIsIssuingToken(false);
      }
    },
    [apiClient, notify, traderId]
  );

  const issueTokenWithWallet = useCallback(
    async (ttlSeconds?: number) => {
      if (typeof window === "undefined" || !window.ethereum) {
        notify("error", "No wallet provider detected in this browser.");
        return;
      }
      const address = walletAddress?.trim();
      if (!address) {
        notify("error", "Connect a wallet before requesting a wallet-signed token.");
        return;
      }
      setIsIssuingToken(true);
      try {
        const challenge = await apiClient.requestWalletChallenge(address);
        const signature = (await window.ethereum.request({
          method: "personal_sign",
          params: [challenge.challenge, address]
        })) as string;
        const response = await apiClient.requestWalletToken({
          address,
          signature,
          ttl_seconds: ttlSeconds
        });
        setToken(response.token);
        setTraderIdState((prev) => prev || address);
        notify("success", "Issued JWT via wallet signature.");
      } catch (error) {
        notify("error", error instanceof Error ? error.message : "Wallet token issuance failed.");
      } finally {
        setIsIssuingToken(false);
      }
    },
    [apiClient, notify, walletAddress]
  );

  const submitOrder = useCallback(
    async (values: OrderFormValues) => {
      const trader = (traderId || walletAddress || "").trim();
      if (!trader) {
        notify("error", "Set a trader ID (wallet address) before placing orders.");
        return;
      }
      if (!token) {
        notify("error", "Paste a valid JWT token to authenticate order submissions.");
        return;
      }
      const quantity = Math.trunc(values.quantity);
      const price = values.orderType === "limit" && values.price ? Math.trunc(values.price) : undefined;

      if (Number.isNaN(quantity) || quantity <= 0) {
        notify("error", "Quantity must be a positive integer.");
        return;
      }
      if (values.orderType === "limit" && (!price || price <= 0)) {
        notify("error", "Limit orders require a positive price.");
        return;
      }

      setIsSubmittingOrder(true);
      try {
        const response = await apiClient.createOrder({
          trader_id: trader,
          base_token: values.baseToken.trim().toUpperCase(),
          quote_token: values.quoteToken.trim().toUpperCase(),
          side: values.side,
          order_type: values.orderType,
          price,
          quantity
        });
        setLastOrderId(response.order_id);
        notify("success", response.message ?? `Order ${response.order_id} accepted.`);
        await Promise.allSettled([refreshMarketData(), fetchTrades(trader, true)]);
      } catch (error) {
        notify("error", error instanceof Error ? error.message : "Order submission failed.");
        throw error;
      } finally {
        setIsSubmittingOrder(false);
      }
    },
    [apiClient, fetchTrades, notify, refreshMarketData, token, traderId, walletAddress]
  );

  return {
    status,
    apiBaseUrl,
    setApiBaseUrl,
    wallet: {
      address: walletAddress,
      traderId,
      token
    },
    setTraderId,
    setAuthToken,
    connectWallet,
    market,
    refreshMarketData,
    depth,
    refreshDepth,
    trades,
    loadTrades,
    isRefreshingMarket,
    isFetchingTrades,
    isSubmittingOrder,
    isIssuingToken,
    issueTokenWithSecret,
    issueTokenWithWallet,
    submitOrder,
    lastOrderId,
    notification,
    dismissNotification
  };
}

function shorten(value: string) {
  if (value.length <= 12) {
    return value;
  }
  return `${value.slice(0, 6)}…${value.slice(-4)}`;
}

function buildDepthWsUrl(baseUrl: string, levels: number): string | null {
  try {
    const url = new URL(baseUrl);
    url.protocol = url.protocol === "https:" ? "wss:" : "ws:";
    url.pathname = "/ws/depth";
    url.search = new URLSearchParams({ levels: String(levels) }).toString();
    return url.toString();
  } catch {
    return null;
  }
}
