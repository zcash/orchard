window.BENCHMARK_DATA = {
  "lastUpdate": 1790004876976,
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
          "id": "2929d3b7114b4eb3ae36f1d705531f124ac450ef",
          "message": "Merge pull request #560 from nullcopy/pczt-action-decryption\n\npczt: add trial decryption and output recovery to Action",
          "timestamp": "2026-09-21T10:23:14-05:00",
          "tree_id": "8f1f0a75b86804fed486cb501ade28fc6a247fcc",
          "url": "https://github.com/zcash/orchard/commit/2929d3b7114b4eb3ae36f1d705531f124ac450ef"
        },
        "date": 1790004875865,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2549952013,
            "range": "± 27392959",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2514479582,
            "range": "± 33567546",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 3627419236,
            "range": "± 19314606",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 4717468757,
            "range": "± 51737220",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 19208408,
            "range": "± 567671",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 19130657,
            "range": "± 139090",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 22231432,
            "range": "± 161016",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 25052598,
            "range": "± 172330",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1260356,
            "range": "± 9391",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 103735,
            "range": "± 825",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1256381,
            "range": "± 14641",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1087847267,
            "range": "± 2542474",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 12677957,
            "range": "± 156692",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 1118614,
            "range": "± 12597",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 12632013,
            "range": "± 25745",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 1084059,
            "range": "± 6449",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 63254672,
            "range": "± 144254",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 5515259,
            "range": "± 15057",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 63060073,
            "range": "± 109868",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 5343537,
            "range": "± 9617",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 126513618,
            "range": "± 409816",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 11009923,
            "range": "± 30773",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 126158328,
            "range": "± 1097161",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 10669179,
            "range": "± 23153",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 380290,
            "range": "± 4296",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 410604,
            "range": "± 919",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}