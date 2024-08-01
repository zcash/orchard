window.BENCHMARK_DATA = {
  "lastUpdate": 1722533052349,
  "repoUrl": "https://github.com/zcash/orchard",
  "entries": {
    "Orchard Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "kris@nutty.land",
            "name": "Kris Nuttycombe",
            "username": "nuttycom"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d4136c645beb2d00f86c9b5ce3a94e42b10666eb",
          "message": "Merge pull request #428 from pacu/ak-from-bytes\n\nAllow SpendValidatingKey to be constructed from bytes",
          "timestamp": "2024-08-01T11:09:16-06:00",
          "tree_id": "26e57359f638e1265cfa8b6fd6e156f5046febf2",
          "url": "https://github.com/zcash/orchard/commit/d4136c645beb2d00f86c9b5ce3a94e42b10666eb"
        },
        "date": 1722533051441,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2897983027,
            "range": "± 24880859",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2892734787,
            "range": "± 23330223",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4145459470,
            "range": "± 8660284",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5390930457,
            "range": "± 11828597",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 24943692,
            "range": "± 533406",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 25257712,
            "range": "± 537388",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28180621,
            "range": "± 469640",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31886878,
            "range": "± 393877",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1522933,
            "range": "± 11453",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 127568,
            "range": "± 252",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1519551,
            "range": "± 6418",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1338133035,
            "range": "± 1162628",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16116687,
            "range": "± 243616",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2187467,
            "range": "± 8673",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16069783,
            "range": "± 33641",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2150994,
            "range": "± 8632",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 80570470,
            "range": "± 376648",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10879520,
            "range": "± 32205",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 80310636,
            "range": "± 183266",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10700475,
            "range": "± 37940",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 161092395,
            "range": "± 324022",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21756552,
            "range": "± 74589",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 160591670,
            "range": "± 370529",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 21388925,
            "range": "± 60388",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 468824,
            "range": "± 1012",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 503174,
            "range": "± 852",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}