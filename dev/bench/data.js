window.BENCHMARK_DATA = {
  "lastUpdate": 1709247283564,
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
      },
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
          "id": "1db974141bdd2dd921a515b1c66df7a7fad91761",
          "message": "Merge pull request #420 from zcash/release-0.7.1\n\nRelease 0.7.1",
          "timestamp": "2024-02-29T22:38:17Z",
          "tree_id": "b541bff350254111bd67f50dfc57a747402bcd89",
          "url": "https://github.com/zcash/orchard/commit/1db974141bdd2dd921a515b1c66df7a7fad91761"
        },
        "date": 1709247282556,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 3095173418,
            "range": "± 154502704",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 3093001434,
            "range": "± 37552182",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 4401230379,
            "range": "± 19467536",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5697329977,
            "range": "± 40142949",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 27546419,
            "range": "± 2192851",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 25473258,
            "range": "± 1062186",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 28544281,
            "range": "± 932869",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 31580663,
            "range": "± 434553",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1529869,
            "range": "± 12894",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 127483,
            "range": "± 256",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1526487,
            "range": "± 110337",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1358545556,
            "range": "± 1989067",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 16155479,
            "range": "± 29205",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2177244,
            "range": "± 5234",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 16146053,
            "range": "± 18132",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2136204,
            "range": "± 6869",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 80766261,
            "range": "± 336059",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10824429,
            "range": "± 28154",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 80572233,
            "range": "± 211271",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10621781,
            "range": "± 65902",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 161479547,
            "range": "± 443112",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21679543,
            "range": "± 74931",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 161258151,
            "range": "± 460048",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 21243706,
            "range": "± 78283",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 465436,
            "range": "± 35809",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 500855,
            "range": "± 2222",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}