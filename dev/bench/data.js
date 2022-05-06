window.BENCHMARK_DATA = {
  "lastUpdate": 1651867711518,
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
          "id": "15007026b1181b14f45c5c8cbf32e4fe86bc8d6b",
          "message": "Merge pull request #325 from zcash/fix-lints\n\nFix lints",
          "timestamp": "2022-05-06T20:51:43+01:00",
          "tree_id": "b7d19f5e78c7d77ff0599cbbaa928cc1937346c5",
          "url": "https://github.com/zcash/orchard/commit/15007026b1181b14f45c5c8cbf32e4fe86bc8d6b"
        },
        "date": 1651867710003,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 4072868289,
            "range": "± 81707487",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 3681989978,
            "range": "± 183845646",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 5632724642,
            "range": "± 302597306",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 7417207082,
            "range": "± 217715416",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 33603572,
            "range": "± 1306038",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 33259990,
            "range": "± 670431",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 37296117,
            "range": "± 1851115",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 40893510,
            "range": "± 2278571",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1046349,
            "range": "± 966",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 132418,
            "range": "± 139",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1042792,
            "range": "± 606",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 135648916,
            "range": "± 40397",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 20596925,
            "range": "± 12221",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2336193,
            "range": "± 1375",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 20539344,
            "range": "± 9854",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2029752,
            "range": "± 795",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 90793090,
            "range": "± 40026",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10261780,
            "range": "± 3389",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 102625239,
            "range": "± 53449",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 11447042,
            "range": "± 5642",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 205915520,
            "range": "± 92580",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 23242464,
            "range": "± 6342",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 181025937,
            "range": "± 109167",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 22878132,
            "range": "± 15214",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 493608,
            "range": "± 2760",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 546950,
            "range": "± 234",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}