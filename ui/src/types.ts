/**
 * FurlPay Real-Time Search Loading & Streaming Types
 */

export type SearchLoadingState =
  | 'idle'
  | 'initializing'
  | 'index_lookup'
  | 'hybrid_retrieval'
  | 'verification'
  | 'ranking'
  | 'streaming'
  | 'completed'
  | 'no_matches'
  | 'error';

export type SearchMode = 'code' | 'web' | 'url' | 'hybrid';

export interface EntityClassification {
  queryClass: string;
  detectedEntity?: string;
  network?: string;
  confidence: number;
}

export interface SearchStateMessage {
  state: SearchLoadingState;
  uiMessage: string;
}

export const STATE_UI_MESSAGES: Record<SearchLoadingState, string> = {
  idle: 'Search your codebase…',
  initializing: 'Preparing search…',
  index_lookup: 'Searching indexed files…',
  hybrid_retrieval: 'Finding relevant code…',
  verification: 'Verifying matches…',
  ranking: 'Ranking results…',
  streaming: 'Loading results…',
  completed: 'Results ready',
  no_matches: 'No relevant results found',
  error: 'Search failed — retry',
};

export interface SearchStep {
  id: number;
  title: string;
  description: string;
  status: 'pending' | 'in_progress' | 'completed';
}

export interface SearchResultItem {
  id: string;
  filePath: string;
  title: string;
  language: string;
  lineNumber: number;
  matchLength: number;
  score: number;
  snippet: string;
  verified: boolean;
  retrievalLayer: string;
  timestampMs: number;
  timestampUs?: number;
}

export interface SearchProgressTelemetry {
  ttfbMs?: number;
  ttfeMs?: number;
  ttfmMs?: number;
  ttfvMs?: number;
  ttfpMs?: number;
  ttfCompleteMs?: number;
  totalDurationMs?: number;
  totalMatchesFound: number;
  totalVerified: number;
}
