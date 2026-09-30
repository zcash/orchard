window.BENCHMARK_DATA = {
  "lastUpdate": 1790800448263,
  "repoUrl": "https://github.com/zcash/orchard",
  "entries": {
    "Orchard Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "john@coldnoise.net",
            "name": "John",
            "username": "nullcopy"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "250c248411e988b581f427fcadcb6a30297f5584",
          "message": "Merge pull request #569 from zcash/pre-release/0.16.0\n\nUpdate dependencies in preparation for release 0.16.0",
          "timestamp": "2026-09-30T15:23:55-05:00",
          "tree_id": "3cc486a7f5ba81c65fdeb0316a8b2703f1521cd6",
          "url": "https://github.com/zcash/orchard/commit/250c248411e988b581f427fcadcb6a30297f5584"
        },
        "date": 1790800446244,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2035713920,
            "range": "± 15847081",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2032984228,
            "range": "± 3852412",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 2912995869,
            "range": "± 4448408",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 3816645786,
            "range": "± 26684818",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 15728961,
            "range": "± 102630",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 15745717,
            "range": "± 99930",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 18238648,
            "range": "± 121267",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 20447759,
            "range": "± 289537",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 982411,
            "range": "± 2000",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 89485,
            "range": "± 162",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 980021,
            "range": "± 2069",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 951222706,
            "range": "± 322344",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 9884025,
            "range": "± 17480",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 960619,
            "range": "± 1013",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 9868064,
            "range": "± 11820",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 929379,
            "range": "± 2558",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 49359384,
            "range": "± 38813",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 4736685,
            "range": "± 10724",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 49271721,
            "range": "± 118184",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 4579592,
            "range": "± 5754",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 98687957,
            "range": "± 94717",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 9455841,
            "range": "± 12177",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 98567794,
            "range": "± 106525",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 9154973,
            "range": "± 27825",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}