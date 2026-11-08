import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  scenarios: {
    search_load: {
      executor: 'constant-vus',
      vus: 100,
      duration: '2m',
    },
  },
  thresholds: {
    http_req_failed: ['rate<0.01'],
    http_req_duration: [
      'p(95)<100', // Target: p95 < 100ms
      'p(99)<250', // Target: p99 < 250ms
    ],
  },
};

export default function () {
  const queries = [
    'authentication middleware',
    'handlePayment',
    'verifyJwtToken',
    'PaymentService',
    'reconcile_settlement',
    'auth.*middleware',
    'DatabaseConfig',
  ];
  const query = queries[Math.floor(Math.random() * queries.length)];

  const response = http.post(
    'http://localhost:3000/api/search',
    JSON.stringify({
      query: query,
      max_results: 10,
    }),
    {
      headers: {
        'Content-Type': 'application/json',
      },
    }
  );

  check(response, {
    'status is 200': (r) => r.status === 200,
    'has results': (r) => {
      try {
        const body = JSON.parse(r.body);
        return body && (body.results !== undefined || body.hits !== undefined);
      } catch (e) {
        return false;
      }
    },
  });

  sleep(0.05);
}
