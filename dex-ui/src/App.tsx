import { useDex } from "./hooks/useDex";
import "./styles/app.css";

const App = () => {
  const { status, lastMessage, bestBid, bestAsk, lastTrades, error, runDemo, initialize } =
    useDex();

  return (
    <main className="app">
      <header className="app__header">
        <h1>DEX-OS Web UI (WASM)</h1>
        <p className="app__status">
          Status: <span data-state={status}>{status.toUpperCase()}</span>
        </p>
        <button type="button" className="ghost" onClick={() => initialize()}>
          Reload WASM
        </button>
      </header>

      <section className="app__body">
        <div className="card">
          <h2>Quick Demo</h2>
          <p>{lastMessage}</p>
          <button type="button" onClick={() => runDemo()} disabled={status === "loading"}>
            Run sample order match
          </button>
          {error && (
            <p className="app__error">
              <strong>Error:</strong> {error}
            </p>
          )}
        </div>

        <div className="grid">
          <article className="card">
            <h3>Orderbook Snapshot</h3>
            <div className="metrics">
              <p>
                Best Bid:{" "}
                <span className="metric">
                  {typeof bestBid === "number" ? `$${bestBid.toLocaleString()}` : "N/A"}
                </span>
              </p>
              <p>
                Best Ask:{" "}
                <span className="metric">
                  {typeof bestAsk === "number" ? `$${bestAsk.toLocaleString()}` : "N/A"}
                </span>
              </p>
            </div>
          </article>

          <article className="card">
            <h3>Recent Trades</h3>
            {lastTrades.length === 0 ? (
              <p className="muted">Trigger the demo to populate trades.</p>
            ) : (
              <ul className="trades">
                {lastTrades.map((trade) => (
                  <li key={trade.id}>
                    <span>Trade #{trade.id}</span>
                    <span>
                      {trade.quantity} units @ ${trade.price.toLocaleString()}
                    </span>
                  </li>
                ))}
              </ul>
            )}
          </article>
        </div>
      </section>

      <footer className="app__footer">
        <p>
          Build the wasm bundle with <code>npm run wasm:build</code> before starting the dev
          server. This UI uses the bindings generated from <code>dex-wasm</code>.
        </p>
      </footer>
    </main>
  );
};

export default App;
