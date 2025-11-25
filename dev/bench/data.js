window.BENCHMARK_DATA = {
  "lastUpdate": 1764101372071,
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
          "id": "a799082eb7085b1d109a44e6d6b4db35dd6f5a1c",
          "message": "Merge pull request #472 from zcash/pczt-external-sigs\n\npczt: Support applying external spendAuthSigs to Spends",
          "timestamp": "2025-11-25T12:56:23-07:00",
          "tree_id": "25049917ca66be2461eb2a29c4e0f87a6673ab3c",
          "url": "https://github.com/zcash/orchard/commit/a799082eb7085b1d109a44e6d6b4db35dd6f5a1c"
        },
        "date": 1764101370524,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2851949676,
            "range": "± 215990361",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2844979503,
            "range": "± 4360477",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4081175568,
            "range": "± 32707261",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5307796448,
            "range": "± 16008926",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 24151530,
            "range": "± 962203",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 24238649,
            "range": "± 193741",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 27761163,
            "range": "± 290458",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31141501,
            "range": "± 246663",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1482268,
            "range": "± 29592",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 126008,
            "range": "± 1069",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1479446,
            "range": "± 9511",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1348923845,
            "range": "± 7199354",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 15650783,
            "range": "± 486431",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2137352,
            "range": "± 7784",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 15620356,
            "range": "± 22455",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2103333,
            "range": "± 61282",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 78181507,
            "range": "± 145072",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10628742,
            "range": "± 110836",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 78039991,
            "range": "± 1811075",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10459636,
            "range": "± 14582",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 156280370,
            "range": "± 974063",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21242895,
            "range": "± 273442",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 155995102,
            "range": "± 2405269",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 20898721,
            "range": "± 37867",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 464013,
            "range": "± 3282",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 488093,
            "range": "± 5502",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}