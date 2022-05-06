window.BENCHMARK_DATA = {
  "lastUpdate": 1651865752916,
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
          "id": "c71de550ed113a2726da691aef89228b7cbd37d5",
          "message": "Merge pull request #313 from zcash/full-width-var-base-mul\n\nUse new halo2 `FixedPoint` API.",
          "timestamp": "2022-05-06T20:19:03+01:00",
          "tree_id": "78577deec8d414d22a506f6e3d752668c13eb07b",
          "url": "https://github.com/zcash/orchard/commit/c71de550ed113a2726da691aef89228b7cbd37d5"
        },
        "date": 1651865751339,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 3942565367,
            "range": "± 128199936",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 3942940590,
            "range": "± 9946679",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 5637221427,
            "range": "± 23289344",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 7325088180,
            "range": "± 18593636",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 33014965,
            "range": "± 290545",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 32543235,
            "range": "± 311658",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 36278360,
            "range": "± 248071",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 39483278,
            "range": "± 452914",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1046236,
            "range": "± 410",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 116659,
            "range": "± 58",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 920289,
            "range": "± 465",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 118863369,
            "range": "± 41975",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 18137944,
            "range": "± 8876",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2059336,
            "range": "± 1205",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 20503524,
            "range": "± 122142",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2027892,
            "range": "± 1403",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 90658181,
            "range": "± 29475",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10250039,
            "range": "± 5167",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 90428756,
            "range": "± 38486",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10091027,
            "range": "± 121236",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 181319661,
            "range": "± 55791",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 20491828,
            "range": "± 14039",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 180813113,
            "range": "± 49257",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 20163007,
            "range": "± 8918",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 435663,
            "range": "± 4244",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 482560,
            "range": "± 417",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}