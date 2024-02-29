window.BENCHMARK_DATA = {
  "lastUpdate": 1709246420598,
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
          "id": "2a312c0cf161be7fe6d706de4463ce32f92de8bb",
          "message": "Merge pull request #419 from nuttycom/nullifier_ct_eq\n\nAdditions needed for Orchard batch scanning",
          "timestamp": "2024-02-29T22:24:02Z",
          "tree_id": "76719346c51a3d992e37019351f2687319d3a172",
          "url": "https://github.com/zcash/orchard/commit/2a312c0cf161be7fe6d706de4463ce32f92de8bb"
        },
        "date": 1709246419179,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2912718073,
            "range": "± 18144682",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2890833296,
            "range": "± 11307597",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4137357455,
            "range": "± 22295821",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5420055263,
            "range": "± 24823399",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 25070685,
            "range": "± 412571",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 25064948,
            "range": "± 395529",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28228520,
            "range": "± 561575",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31787045,
            "range": "± 314850",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1525135,
            "range": "± 9616",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 128083,
            "range": "± 296",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1523744,
            "range": "± 12335",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1350335546,
            "range": "± 690023",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16098548,
            "range": "± 55529",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2163800,
            "range": "± 6058",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16088202,
            "range": "± 54730",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2133319,
            "range": "± 12433",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 80404415,
            "range": "± 881874",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10752196,
            "range": "± 36044",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 80366615,
            "range": "± 253875",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10611977,
            "range": "± 28604",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 160864435,
            "range": "± 1302598",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21502896,
            "range": "± 65771",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 160811566,
            "range": "± 641865",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 21200158,
            "range": "± 80355",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 465601,
            "range": "± 1353",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 501134,
            "range": "± 1710",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}