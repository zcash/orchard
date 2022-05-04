window.BENCHMARK_DATA = {
  "lastUpdate": 1651679119320,
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
          "id": "39495dc94b90380caccf32bbb4190685585e2804",
          "message": "Merge pull request #319 from zcash/279-update-builder-docs\n\nUpdate `Builder::build` docs",
          "timestamp": "2022-05-04T16:21:07+01:00",
          "tree_id": "080582becccc62d8e0db0cce1ae07d14233521d4",
          "url": "https://github.com/zcash/orchard/commit/39495dc94b90380caccf32bbb4190685585e2804"
        },
        "date": 1651678606380,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 5252825229,
            "range": "± 172298188",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 5419327447,
            "range": "± 126606233",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 7722071197,
            "range": "± 74802883",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 9743272283,
            "range": "± 206718056",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 42146321,
            "range": "± 1698181",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 43573703,
            "range": "± 2506332",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 46715743,
            "range": "± 1774078",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 51971171,
            "range": "± 2317565",
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
          "id": "dc89386df14efc47d57f95ab4bed6ddd134d5529",
          "message": "Merge pull request #320 from zcash/243-compact-action-nullifier\n\nAdd nullifier field to `CompactAction`",
          "timestamp": "2022-05-04T16:27:03+01:00",
          "tree_id": "c74972c406e2ebd480646594e268582552d51a4b",
          "url": "https://github.com/zcash/orchard/commit/dc89386df14efc47d57f95ab4bed6ddd134d5529"
        },
        "date": 1651679118103,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 4628038034,
            "range": "± 35673476",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 4623520025,
            "range": "± 10153190",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 6617421522,
            "range": "± 13638042",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 8613461096,
            "range": "± 16080522",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 37999758,
            "range": "± 360640",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 38052759,
            "range": "± 161611",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 42462875,
            "range": "± 284307",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 46448394,
            "range": "± 2230933",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1241786,
            "range": "± 1194",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 157124,
            "range": "± 164",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1236772,
            "range": "± 4477",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 161453660,
            "range": "± 331043",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 24419178,
            "range": "± 52748",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2756091,
            "range": "± 2441",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 24417861,
            "range": "± 20978",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2710700,
            "range": "± 2260",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 122020860,
            "range": "± 63016",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 13715731,
            "range": "± 7857",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 122001110,
            "range": "± 83107",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 13494271,
            "range": "± 15338",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 244003137,
            "range": "± 413487",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 27427762,
            "range": "± 18723",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 244080679,
            "range": "± 457638",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 26979011,
            "range": "± 14349",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 578366,
            "range": "± 1400",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 639759,
            "range": "± 433",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}