window.BENCHMARK_DATA = {
  "lastUpdate": 1772560969871,
  "repoUrl": "https://github.com/zcash/orchard",
  "entries": {
    "Orchard Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "jack@electriccoin.co",
            "name": "Jack Grigg",
            "username": "str4d"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "17f835d06587f2cd69ef5931bce371d57848e524",
          "message": "Merge pull request #474 from zcash/release-0.12.0\n\norchard 0.12.0",
          "timestamp": "2025-12-05T17:11:44Z",
          "tree_id": "873cade7725160afc8d56a7146cc4033df64d3d2",
          "url": "https://github.com/zcash/orchard/commit/17f835d06587f2cd69ef5931bce371d57848e524"
        },
        "date": 1764955465897,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2683994910,
            "range": "± 202586591",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2676573332,
            "range": "± 4570160",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 3858343708,
            "range": "± 4769882",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5017598332,
            "range": "± 15060267",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 20953219,
            "range": "± 128159",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 21085723,
            "range": "± 183708",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 24333441,
            "range": "± 213497",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 27653348,
            "range": "± 249032",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1471043,
            "range": "± 7367",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 125458,
            "range": "± 177",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1467663,
            "range": "± 4915",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1334335416,
            "range": "± 1482164",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 15536565,
            "range": "± 28427",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2130053,
            "range": "± 3995",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 15502560,
            "range": "± 65412",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2094656,
            "range": "± 4483",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 77625235,
            "range": "± 166587",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10595083,
            "range": "± 13872",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 77458547,
            "range": "± 136442",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10420664,
            "range": "± 20945",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 155265105,
            "range": "± 140015",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21175560,
            "range": "± 30719",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 154908416,
            "range": "± 118853",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 20828646,
            "range": "± 33069",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 461245,
            "range": "± 1241",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 488274,
            "range": "± 794",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "ecc1fb64de9e8c5ec66d70ee3f0788aeb791e5b9",
          "message": "Merge pull request #478 from zcash/fix-ci\n\nCI: Use pinned dependencies where possible for `build-nostd`",
          "timestamp": "2026-03-02T07:47:54-08:00",
          "tree_id": "f1dc0a25ea70b5682694708b8c6dc6b241698cd9",
          "url": "https://github.com/zcash/orchard/commit/ecc1fb64de9e8c5ec66d70ee3f0788aeb791e5b9"
        },
        "date": 1772467239887,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2723552458,
            "range": "± 129895037",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2728436643,
            "range": "± 25544201",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 3900093232,
            "range": "± 14547304",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5063485695,
            "range": "± 16588522",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 21454506,
            "range": "± 166034",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 21421562,
            "range": "± 401138",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 24700720,
            "range": "± 168913",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 27985504,
            "range": "± 216122",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1486435,
            "range": "± 13304",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 125501,
            "range": "± 401",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1482871,
            "range": "± 7108",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1333611402,
            "range": "± 7472509",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 15689412,
            "range": "± 26550",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2126916,
            "range": "± 12313",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 15669847,
            "range": "± 212424",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2092666,
            "range": "± 3532",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 78437449,
            "range": "± 111441",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10578266,
            "range": "± 16092",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 78324476,
            "range": "± 484789",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10412483,
            "range": "± 17622",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 156885609,
            "range": "± 500524",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21157350,
            "range": "± 115732",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 156473932,
            "range": "± 200049",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 20795905,
            "range": "± 25228",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 461792,
            "range": "± 1648",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 487898,
            "range": "± 1552",
            "unit": "ns/iter"
          }
        ]
      },
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
          "id": "64599b81ef4fee59c41a5b98619e27a8be38f953",
          "message": "Merge pull request #477 from zcash/pczt_extract_reference\n\nMake pczt::Bundle::extract take `self` by reference.",
          "timestamp": "2026-03-03T09:50:03-08:00",
          "tree_id": "32086be78565bb131d6647582abacf35963dccfe",
          "url": "https://github.com/zcash/orchard/commit/64599b81ef4fee59c41a5b98619e27a8be38f953"
        },
        "date": 1772560967806,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2693922107,
            "range": "± 218171675",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2688710709,
            "range": "± 4851594",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 3877757545,
            "range": "± 16647351",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 5056167009,
            "range": "± 42721806",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 21296559,
            "range": "± 177753",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 21257065,
            "range": "± 178415",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 24683070,
            "range": "± 214750",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 27925755,
            "range": "± 203720",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1477032,
            "range": "± 12824",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 125844,
            "range": "± 269",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1473990,
            "range": "± 9685",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1328095518,
            "range": "± 2352143",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 15604843,
            "range": "± 69108",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2135190,
            "range": "± 9310",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 15583445,
            "range": "± 34454",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2100841,
            "range": "± 2942",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 78004430,
            "range": "± 1510333",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10621492,
            "range": "± 12062",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 77872506,
            "range": "± 538815",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10449590,
            "range": "± 138854",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 155927646,
            "range": "± 1110087",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21235811,
            "range": "± 377940",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 155713591,
            "range": "± 161096",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 20883885,
            "range": "± 22650",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 461812,
            "range": "± 2503",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 488399,
            "range": "± 2172",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}