import React from 'react';
import { useFurlPaySearch } from './useFurlPaySearch';
import { SearchMode, SearchStep, SearchResultItem } from './types';

export const FurlPaySearch: React.FC = () => {
  const {
    query,
    setQuery,
    mode,
    setMode,
    loadingState,
    uiMessage,
    results,
    steps,
    classification,
    telemetry,
    executeSearch,
    cancelSearch,
  } = useFurlPaySearch();

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const val = e.target.value;
    setQuery(val);
    executeSearch(val, false);
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      executeSearch(query, true);
    }
  };

  const handleClear = () => {
    setQuery('');
    cancelSearch();
  };

  const isSearching =
    loadingState === 'initializing' ||
    loadingState === 'index_lookup' ||
    loadingState === 'hybrid_retrieval' ||
    loadingState === 'verification' ||
    loadingState === 'ranking' ||
    loadingState === 'streaming';

  const getStatusBadge = () => {
    if (isSearching) {
      return (
        <span className="status-badge working">
          <div className="square-loader" />
          <span>Working…</span>
          <div className="dots-loader">
            <span />
            <span />
            <span />
          </div>
        </span>
      );
    }
    switch (loadingState) {
      case 'idle':
        return <span className="status-badge idle">Ready</span>;
      case 'completed':
        return <span className="status-badge ready">✓ Results ready</span>;
      case 'no_matches':
        return <span className="status-badge empty">No results found</span>;
      case 'error':
        return <span className="status-badge error">Search failed</span>;
    }
  };

  return (
    <div className="furlpay-search-container">
      {/* Top Header */}
      <header className="furlpay-header">
        <div className="brand-group">
          <div className="logo-gem">FP</div>
          <div>
            <h1 className="brand-title">FurlPay Search</h1>
            <p className="brand-sub">Sub-millisecond AI agent retrieval runtime</p>
          </div>
        </div>
        <div className="live-pill">
          <span className="pulse-dot"></span>
          Live search
        </div>
      </header>

      {/* Mode Selectors */}
      <div className="mode-tabs">
        {(['code', 'web', 'url', 'hybrid'] as SearchMode[]).map((m) => (
          <button
            key={m}
            className={`mode-tab ${mode === m ? 'active' : ''}`}
            onClick={() => setMode(m)}
          >
            {m === 'code' ? 'Local Code' : m === 'web' ? 'Public Web' : m === 'url' ? 'Direct URL' : 'Hybrid'}
          </button>
        ))}
      </div>

      {/* Search Input Bar */}
      <div className="search-box-wrapper">
        <div className="search-box">
          <svg
            className="search-icon"
            width="20"
            height="20"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
          >
            <circle cx="11" cy="11" r="8" />
            <line x1="21" y1="21" x2="16.65" y2="16.65" />
          </svg>
          <input
            type="text"
            className="search-input"
            value={query}
            onChange={handleInputChange}
            onKeyDown={handleKeyDown}
            placeholder="Search code, tron wallet address, tokens (e.g. tron, TRX, TXYZ...)"
            autoFocus
            spellCheck="false"
          />
          {query.length > 0 && (
            <button className="clear-btn visible" onClick={handleClear} title="Clear search">
              ✕
            </button>
          )}
          <button
            className={`submit-arrow ${query.length > 0 ? 'active' : ''}`}
            onClick={() => executeSearch(query, true)}
            title="Execute search (Enter)"
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2.5"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <line x1="5" y1="12" x2="19" y2="12" />
              <polyline points="12 5 19 12 12 19" />
            </svg>
          </button>
          {getStatusBadge()}
        </div>
        <div className="loading-caption">
          <span>{uiMessage}</span>
          {classification && (
            <span className={`entity-tag ${classification.network ? classification.network.toLowerCase() : 'token'}`}>
              ✦ {classification.network || classification.detectedEntity || classification.queryClass}
            </span>
          )}
        </div>
      </div>

      {/* Main Grid: Pipeline + Verified Results */}
      <div className="furlpay-grid">
        {/* Step Tracker */}
        <div className="pipeline-card">
          <div className="card-header">
            <h2 className="card-title">Search Pipeline</h2>
            <span className="engine-tag">Rust furlpayd</span>
          </div>

          <div className="steps-container">
            {steps.map((s) => (
              <div
                key={s.id}
                className={`step-item ${
                  s.status === 'completed'
                    ? 'done'
                    : s.status === 'in_progress'
                    ? 'working'
                    : ''
                }`}
              >
                <div className="step-col">
                  <div className="step-bubble">
                    {s.status === 'completed' ? '✓' : s.id}
                  </div>
                  {s.id < 4 && <div className="step-line" />}
                </div>
                <div className="step-body">
                  <div className="step-title">{s.title}</div>
                  <p className="step-desc">{s.description}</p>
                </div>
              </div>
            ))}
          </div>

          {/* Telemetry Footer */}
          <div className="telemetry-bar">
            <div>
              TTFV:{' '}
              <span className="telem-val">
                {telemetry.ttfvMs !== undefined ? `${telemetry.ttfvMs}ms` : '—'}
              </span>
            </div>
            <div>
              Total:{' '}
              <span className="telem-val">
                {telemetry.totalDurationMs !== undefined ? `${telemetry.totalDurationMs}ms` : '—'}
              </span>
            </div>
            <div>
              Hits:{' '}
              <span className="telem-val">
                {telemetry.totalMatchesFound || results.length}
              </span>
            </div>
          </div>
        </div>

        {/* Results List */}
        <div className="results-card">
          <div className="card-header">
            <h2 className="card-title">Verified Results</h2>
            <span className="stream-pill">
              {isSearching ? 'Progressive stream active' : 'Stream idle'}
            </span>
          </div>

          <div className="results-list">
            {results.length === 0 && !isSearching && (
              <div className="empty-state">
                <div className="empty-icon">🔍</div>
                <p>Start typing to search in real time.</p>
                <p style={{ fontSize: '12px', marginTop: '4px', color: 'var(--text-muted)' }}>
                  Try: <code>t</code>, <code>tron</code>, <code>TRX</code>, <code>TokenValidator</code>, or paste an address.
                </p>
              </div>
            )}

            {results.map((item) => (
              <div key={item.id} className="result-card">
                <div className="res-top">
                  <div className="file-ref">
                    <span>📄</span>
                    <span>{item.filePath}</span>
                    {item.lineNumber && (
                      <span style={{ color: 'var(--text-muted)', fontSize: '11px' }}>
                        Line {item.lineNumber}
                      </span>
                    )}
                  </div>
                  <div className="badge-group">
                    <span className="layer-badge">{item.retrievalLayer}</span>
                    <span className="score-badge">
                      {Math.round(item.score * 100)}% match
                    </span>
                  </div>
                </div>

                <div className="res-title">{item.title}</div>
                <pre className="code-block">
                  <code>{item.snippet}</code>
                </pre>

                <div className="res-foot">
                  <span>✓ Verified AST Scope</span>
                  <span>
                    Streamed at {item.timestampUs ? `+${item.timestampUs}µs` : `+${item.timestampMs}ms`}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};
