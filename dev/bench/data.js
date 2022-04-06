window.BENCHMARK_DATA = {
  "lastUpdate": 1649279064820,
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
          "id": "330f492cb0348005a1f9abaffd54fec5b5b50649",
          "message": "Merge pull request #312 from zcash/release-0.1.0-beta.3\n\nRelease 0.1.0-beta.3",
          "timestamp": "2022-04-06T21:48:37+01:00",
          "tree_id": "b91119fca8253b3051b0fe5fbd4e15d090f2846f",
          "url": "https://github.com/zcash/orchard/commit/330f492cb0348005a1f9abaffd54fec5b5b50649"
        },
        "date": 1649279063730,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 4050640119,
            "range": "± 36601336",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 4010699352,
            "range": "± 26424623",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 5739426839,
            "range": "± 24022118",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 6770564989,
            "range": "± 218993343",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 28392582,
            "range": "± 230767",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 28258781,
            "range": "± 524488",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 31437027,
            "range": "± 193102",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 34731222,
            "range": "± 317329",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 912932,
            "range": "± 1180",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 116990,
            "range": "± 114",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 910164,
            "range": "± 1467",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 120093957,
            "range": "± 62104",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 17965085,
            "range": "± 14406",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2065342,
            "range": "± 2195",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 17909579,
            "range": "± 15312",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2033743,
            "range": "± 2175",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 89754808,
            "range": "± 132131",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10278911,
            "range": "± 9768",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 89526695,
            "range": "± 311159",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10116549,
            "range": "± 7738",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 179572969,
            "range": "± 115550",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 20553467,
            "range": "± 18452",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 179121872,
            "range": "± 176494",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 20233332,
            "range": "± 1155249",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 493664,
            "range": "± 233",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 479449,
            "range": "± 541",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}