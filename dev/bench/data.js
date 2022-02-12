window.BENCHMARK_DATA = {
  "lastUpdate": 1644637243522,
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
          "id": "4ae32ef98aef8c209baa4e31c115f42c56dd3886",
          "message": "Merge pull request #282 from zcash/clone-unauthorized-bundle\n\nAdd `Clone` impls to various structs",
          "timestamp": "2022-02-12T03:23:11Z",
          "tree_id": "fab1fea97a9263781e56a70c5c75066637630acb",
          "url": "https://github.com/zcash/orchard/commit/4ae32ef98aef8c209baa4e31c115f42c56dd3886"
        },
        "date": 1644637242460,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 4155286772,
            "range": "± 25307502",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 4144255505,
            "range": "± 15763154",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 5927370561,
            "range": "± 24547605",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 7742585552,
            "range": "± 28381662",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 32460396,
            "range": "± 324271",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 32309244,
            "range": "± 701924",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 35960716,
            "range": "± 269298",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 39916468,
            "range": "± 225215",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1051257,
            "range": "± 552",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 133030,
            "range": "± 89",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1048725,
            "range": "± 818",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 137972842,
            "range": "± 63353",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 20696152,
            "range": "± 56512",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2361098,
            "range": "± 1523",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 20722869,
            "range": "± 13685",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2324532,
            "range": "± 829",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 103443417,
            "range": "± 47173",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 11748302,
            "range": "± 50277",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 103568752,
            "range": "± 48121",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 11566150,
            "range": "± 5811",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 206903091,
            "range": "± 314321",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 23484301,
            "range": "± 13977",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 207098923,
            "range": "± 76880",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 23123539,
            "range": "± 16701",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 498613,
            "range": "± 294",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 546505,
            "range": "± 283",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}