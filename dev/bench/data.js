window.BENCHMARK_DATA = {
  "lastUpdate": 1651188833703,
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
          "id": "b6a69cdfdcd72562373853daafb0333b317f656e",
          "message": "Merge pull request #316 from zcash/no-std-prep\n\nPreparations for `no-std` support",
          "timestamp": "2022-04-29T00:13:49+01:00",
          "tree_id": "16c2ed6c459f889eecd415bac26bfec6a14425f5",
          "url": "https://github.com/zcash/orchard/commit/b6a69cdfdcd72562373853daafb0333b317f656e"
        },
        "date": 1651188832507,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 5247907038,
            "range": "± 103589755",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 5239425097,
            "range": "± 35702551",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 7487298106,
            "range": "± 42569504",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 9785778255,
            "range": "± 50717059",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 40698366,
            "range": "± 1547764",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 41199891,
            "range": "± 1724040",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 45442671,
            "range": "± 9771635",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 50306456,
            "range": "± 3264935",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1275539,
            "range": "± 35454",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 162484,
            "range": "± 4540",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1275792,
            "range": "± 44886",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 162803053,
            "range": "± 3631704",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 25238479,
            "range": "± 1021816",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2843761,
            "range": "± 84739",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 25208628,
            "range": "± 1120330",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2805305,
            "range": "± 117186",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 125955886,
            "range": "± 3027860",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 14216462,
            "range": "± 654954",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 126071487,
            "range": "± 4873256",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 14014735,
            "range": "± 410078",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 255080198,
            "range": "± 6442160",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 28301824,
            "range": "± 954287",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 254325695,
            "range": "± 4716459",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 27956072,
            "range": "± 1085746",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 601089,
            "range": "± 63342",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 667989,
            "range": "± 29015",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}