window.BENCHMARK_DATA = {
  "lastUpdate": 1733952007609,
  "repoUrl": "https://github.com/zcash/orchard",
  "entries": {
    "Orchard Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "daira@jacaranda.org",
            "name": "Daira-Emma Hopwood",
            "username": "daira"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5c451beb05a10337a57a7fdf279c1dd6a533b805",
          "message": "Merge pull request #440 from zcash/pczt\n\nImplement PCZT support",
          "timestamp": "2024-12-11T21:06:19Z",
          "tree_id": "6b73bb04e5008e3940e717d91e825b8b3e8c67b0",
          "url": "https://github.com/zcash/orchard/commit/5c451beb05a10337a57a7fdf279c1dd6a533b805"
        },
        "date": 1733952006808,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2920864871,
            "range": "± 24855491",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2896613989,
            "range": "± 10731887",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4138769096,
            "range": "± 34287123",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5422622465,
            "range": "± 16733841",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 25052144,
            "range": "± 468451",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 25019198,
            "range": "± 498721",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28355684,
            "range": "± 610007",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31587743,
            "range": "± 477433",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1519619,
            "range": "± 10625",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 126924,
            "range": "± 634",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1515302,
            "range": "± 4562",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1329792369,
            "range": "± 3880483",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16021170,
            "range": "± 15250",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2148844,
            "range": "± 6170",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16002003,
            "range": "± 46984",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2114477,
            "range": "± 9075",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 80032448,
            "range": "± 93365",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10689693,
            "range": "± 102761",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 79949533,
            "range": "± 1807219",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10519049,
            "range": "± 23965",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 160070836,
            "range": "± 249476",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21361955,
            "range": "± 42079",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 159887243,
            "range": "± 545562",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 21024917,
            "range": "± 37053",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 465228,
            "range": "± 929",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 499624,
            "range": "± 1361",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}