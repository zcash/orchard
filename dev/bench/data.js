window.BENCHMARK_DATA = {
  "lastUpdate": 1647979343354,
  "repoUrl": "https://github.com/zcash/orchard",
  "entries": {
    "Orchard Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "str4d",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "43d38f5b97d3b1f88ccc586cd91e76e2443e568d",
          "message": "Merge pull request #301 from zcash/non-consensus-changes-on-branchid-c4cd541e\n\nMerge non-consensus changes",
          "timestamp": "2022-03-22T19:42:35Z",
          "tree_id": "bd43cf41462e3ba3b9f40929138a03a5da90d109",
          "url": "https://github.com/zcash/orchard/commit/43d38f5b97d3b1f88ccc586cd91e76e2443e568d"
        },
        "date": 1647979342249,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 5217935739,
            "range": "± 87115452",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 5227393374,
            "range": "± 68356109",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 7319181848,
            "range": "± 49488015",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 9603085308,
            "range": "± 114870758",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 40209698,
            "range": "± 2214985",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 39710039,
            "range": "± 422584",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 44700445,
            "range": "± 6531479",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 49724631,
            "range": "± 1066174",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1295815,
            "range": "± 580",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 165704,
            "range": "± 1252",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1292841,
            "range": "± 931",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 172318505,
            "range": "± 2470135",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 25480993,
            "range": "± 12693",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2916613,
            "range": "± 1984",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 25439118,
            "range": "± 283402",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2871814,
            "range": "± 1609",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 127362854,
            "range": "± 52304",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 14388256,
            "range": "± 7615",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 127135190,
            "range": "± 51605",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 14287059,
            "range": "± 3260",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 254720375,
            "range": "± 499050",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 29448056,
            "range": "± 720354",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 254258545,
            "range": "± 424984",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 28551857,
            "range": "± 14857",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 625313,
            "range": "± 17222",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 679135,
            "range": "± 287",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}