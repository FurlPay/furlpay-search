import { useState, useRef, useCallback, useEffect } from 'react';
import {
  SearchLoadingState,
  SearchMode,
  STATE_UI_MESSAGES,
  SearchResultItem,
  SearchProgressTelemetry,
  SearchStep,
  EntityClassification,
} from './types';

const INITIAL_STEPS: SearchStep[] = [
  {
    id: 1,
    title: 'Query classification',
    description: 'Determine exact, regex, symbol, blockchain, or hybrid search.',
    status: 'pending',
  },
  {
    id: 2,
    title: 'Parallel retrieval',
    description: 'Search trigram, prefix, AST and symbol indexes.',
    status: 'pending',
  },
  {
    id: 3,
    title: 'Verification & ranking',
    description: 'Verify candidate matches and apply RRF ranking.',
    status: 'pending',
  },
  {
    id: 4,
    title: 'Progressive streaming',
    description: 'Stream verified results immediately as they arrive.',
    status: 'pending',
  },
];

export function useFurlPaySearch(apiBaseUrl = '') {
  const [query, setQuery] = useState('');
  const [mode, setMode] = useState<SearchMode>('code');
  const [loadingState, setLoadingState] = useState<SearchLoadingState>('idle');
  const [results, setResults] = useState<SearchResultItem[]>([]);
  const [steps, setSteps] = useState<SearchStep[]>(INITIAL_STEPS);
  const [classification, setClassification] = useState<EntityClassification | null>(null);
  const [telemetry, setTelemetry] = useState<SearchProgressTelemetry>({
    totalMatchesFound: 0,
    totalVerified: 0,
  });

  const abortControllerRef = useRef<AbortController | null>(null);
  const debounceTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const activeRequestIdRef = useRef<number>(0);

  const cancelSearch = useCallback(() => {
    if (debounceTimerRef.current) {
      clearTimeout(debounceTimerRef.current);
      debounceTimerRef.current = null;
    }
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
      abortControllerRef.current = null;
    }
    setLoadingState('idle');
  }, []);

  const executeSearch = useCallback(
    async (searchQuery: string, isImmediate = false) => {
      if (debounceTimerRef.current) {
        clearTimeout(debounceTimerRef.current);
        debounceTimerRef.current = null;
      }

      const trimmed = searchQuery.trim();
      if (!trimmed) {
        cancelSearch();
        setResults([]);
        setClassification(null);
        setSteps(INITIAL_STEPS);
        setLoadingState('idle');
        return;
      }

      const runFetch = async () => {
        // Cancel prior active network connection
        if (abortControllerRef.current) {
          abortControllerRef.current.abort();
        }

        const controller = new AbortController();
        abortControllerRef.current = controller;
        const requestId = ++activeRequestIdRef.current;

        setLoadingState('initializing');
        setSteps(INITIAL_STEPS.map((s) => (s.id === 1 ? { ...s, status: 'in_progress' } : { ...s, status: 'pending' })));

        const startTime = performance.now();
        const telem: SearchProgressTelemetry = {
          totalMatchesFound: 0,
          totalVerified: 0,
        };

        try {
          const sseUrl = `${apiBaseUrl}/api/v1/search/stream?q=${encodeURIComponent(trimmed)}&mode=${mode}`;
          const response = await fetch(sseUrl, {
            signal: controller.signal,
            headers: { Accept: 'text/event-stream' },
          });

          if (!response.ok || !response.body) {
            if (response.status === 401) {
              setLoadingState('error');
              return;
            }
            throw new Error(`HTTP ${response.status}`);
          }

          telem.ttfbMs = Math.round(performance.now() - startTime);

          const reader = response.body.getReader();
          const decoder = new TextDecoder();
          let buffer = '';
          const resultsMap = new Map<string, SearchResultItem>();

          while (true) {
            const { done, value } = await reader.read();
            if (done) break;
            if (requestId !== activeRequestIdRef.current) return;

            buffer += decoder.decode(value, { stream: true });
            const chunks = buffer.split('\n\n');
            buffer = chunks.pop() || '';

            for (const chunk of chunks) {
              if (!chunk.trim()) continue;
              const eventMatch = chunk.match(/^event:\s*(.+)$/m);
              const dataMatch = chunk.match(/^data:\s*(.+)$/m);

              const eventType = eventMatch ? eventMatch[1].trim() : 'message';
              let data: any = {};
              try {
                data = dataMatch ? JSON.parse(dataMatch[1]) : {};
              } catch (_) {}

              const now = performance.now();

              switch (eventType) {
                case 'search.started':
                  telem.ttfeMs = Math.round(now - startTime);
                  setLoadingState('initializing');
                  break;

                case 'search.classified':
                  setSteps((prev) =>
                    prev.map((s) =>
                      s.id === 1
                        ? { ...s, status: 'completed' }
                        : s.id === 2
                        ? { ...s, status: 'in_progress' }
                        : s
                    )
                  );
                  setClassification({
                    queryClass: data.query_class || '',
                    detectedEntity: data.detected_entity,
                    network: data.query_class?.includes('TRON')
                      ? 'TRON'
                      : data.query_class?.includes('Ethereum')
                      ? 'Ethereum'
                      : data.query_class?.includes('Solana')
                      ? 'Solana'
                      : undefined,
                    confidence: data.confidence || 0.9,
                  });
                  setLoadingState('index_lookup');
                  break;

                case 'search.matches_found':
                  telem.ttfmMs = Math.round(now - startTime);
                  telem.totalMatchesFound = data.count || 0;
                  setSteps((prev) =>
                    prev.map((s) =>
                      s.id === 2
                        ? { ...s, status: 'completed' }
                        : s.id === 3
                        ? { ...s, status: 'in_progress' }
                        : s
                    )
                  );
                  setLoadingState('hybrid_retrieval');
                  break;

                case 'search.verification':
                  if (!telem.ttfvMs) {
                    telem.ttfvMs = Math.round(now - startTime);
                  }
                  telem.totalVerified = data.verified_count || 1;
                  setLoadingState('verification');
                  break;

                case 'search.ranking':
                  setSteps((prev) =>
                    prev.map((s) =>
                      s.id === 3
                        ? { ...s, status: 'completed' }
                        : s.id === 4
                        ? { ...s, status: 'in_progress' }
                        : s
                    )
                  );
                  setLoadingState('ranking');
                  break;

                case 'search.results':
                  if (!telem.ttfpMs) {
                    telem.ttfpMs = Math.round(now - startTime);
                  }
                  if (Array.isArray(data.results)) {
                    data.results.forEach((r: any) => {
                      const item: SearchResultItem = {
                        id: r.id || r.file_path,
                        filePath: r.file_path,
                        title: r.title,
                        language: r.file_path.endsWith('.rs') ? 'Rust' : 'TypeScript',
                        lineNumber: r.line_number || 1,
                        matchLength: 10,
                        score: r.score || 0.9,
                        snippet: r.snippet,
                        verified: r.verified !== false,
                        retrievalLayer: r.retrieval_layer || 'trigram',
                        timestampMs: r.timestamp_ms || Math.round(now - startTime),
                        timestampUs: r.timestamp_us,
                      };
                      resultsMap.set(item.id, item);
                    });
                    setResults(Array.from(resultsMap.values()));
                  }
                  setLoadingState('streaming');
                  break;

                case 'search.empty':
                  setResults([]);
                  setLoadingState('no_matches');
                  break;

                case 'search.completed':
                  telem.ttfCompleteMs = Math.round(now - startTime);
                  telem.totalDurationMs = Math.round(now - startTime);
                  setTelemetry({ ...telem });
                  setSteps((prev) => prev.map((s) => ({ ...s, status: 'completed' })));
                  setResults((current) => {
                    if (current.length === 0) {
                      setLoadingState('no_matches');
                    } else {
                      setLoadingState('completed');
                    }
                    return current;
                  });
                  break;
              }
            }
          }
        } catch (err: any) {
          if (err.name === 'AbortError') return;
          if (requestId === activeRequestIdRef.current) {
            setLoadingState('error');
          }
        }
      };

      if (isImmediate) {
        runFetch();
      } else {
        setLoadingState('initializing');
        debounceTimerRef.current = setTimeout(runFetch, 280);
      }
    },
    [apiBaseUrl, mode, cancelSearch]
  );

  useEffect(() => {
    return () => {
      cancelSearch();
    };
  }, [cancelSearch]);

  return {
    query,
    setQuery,
    mode,
    setMode,
    loadingState,
    uiMessage: STATE_UI_MESSAGES[loadingState],
    results,
    steps,
    classification,
    telemetry,
    executeSearch,
    cancelSearch,
  };
}
