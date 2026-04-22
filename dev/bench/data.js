window.BENCHMARK_DATA = {
  "lastUpdate": 1776892388038,
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
          "id": "b0bf2670e248958c6ce7c1deed466032e0dbd4d9",
          "message": "Merge pull request #482 from zcash/corez_migration\n\nMigrate from yanked `core2` library to `corez`",
          "timestamp": "2026-04-17T14:06:02-06:00",
          "tree_id": "f3d0a82024f726855559de455f646115cbf4e87f",
          "url": "https://github.com/zcash/orchard/commit/b0bf2670e248958c6ce7c1deed466032e0dbd4d9"
        },
        "date": 1776457083836,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2569389895,
            "range": "± 27710516",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2569468549,
            "range": "± 28962309",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 3711114621,
            "range": "± 41349163",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 4826074063,
            "range": "± 30807638",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 20741384,
            "range": "± 396285",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 20694494,
            "range": "± 170306",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 23778574,
            "range": "± 203224",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 26902059,
            "range": "± 212825",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1488655,
            "range": "± 8397",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 124480,
            "range": "± 274",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1482164,
            "range": "± 5484",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1324251045,
            "range": "± 967622",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 15675771,
            "range": "± 23923",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2111448,
            "range": "± 3162",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 15624650,
            "range": "± 25230",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2076876,
            "range": "± 4235",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 78253810,
            "range": "± 127902",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10504222,
            "range": "± 13865",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 78035037,
            "range": "± 110789",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10329359,
            "range": "± 14414",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 156531249,
            "range": "± 227705",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21001640,
            "range": "± 31357",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 156239012,
            "range": "± 252603",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 20648549,
            "range": "± 28366",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 453827,
            "range": "± 1211",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 488916,
            "range": "± 1001",
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
          "id": "efb13500fb6594e2faba26eaf6b3f334060bef8d",
          "message": "Merge pull request #479 from MavenRain/fix/464-depth-overflow\n\nReturn DepthOverflow error instead of panicking at depth 255",
          "timestamp": "2026-04-17T15:08:31-06:00",
          "tree_id": "eea7adeff0d6c89a68e37938a2387f848e4bfef4",
          "url": "https://github.com/zcash/orchard/commit/efb13500fb6594e2faba26eaf6b3f334060bef8d"
        },
        "date": 1776460839050,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2625820002,
            "range": "± 14382570",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2589710482,
            "range": "± 6797812",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 3741807984,
            "range": "± 16236437",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 4873451639,
            "range": "± 36465027",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 20909323,
            "range": "± 232543",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 20794933,
            "range": "± 207843",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 24103177,
            "range": "± 162833",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 27061161,
            "range": "± 749316",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1485544,
            "range": "± 8059",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 124486,
            "range": "± 298",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1481420,
            "range": "± 22804",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1323980271,
            "range": "± 4579379",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 15687223,
            "range": "± 57985",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2105017,
            "range": "± 3303",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 15660784,
            "range": "± 36006",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2068851,
            "range": "± 2529",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 78384828,
            "range": "± 157982",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10480087,
            "range": "± 25083",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 78225124,
            "range": "± 448255",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10295121,
            "range": "± 15768",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 156636117,
            "range": "± 193380",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 20937721,
            "range": "± 76640",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 156405853,
            "range": "± 734754",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 20622063,
            "range": "± 67021",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 453283,
            "range": "± 2106",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 488698,
            "range": "± 5322",
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
          "id": "7e90bf2a2c1c6d766496585a045ca110d6b8f1d8",
          "message": "Merge pull request #480 from ebfull/orchard_internal\n\nSplit orchard into `orchard_internal` (wide visibility) + `orchard` (API-preserving shim)",
          "timestamp": "2026-04-20T09:02:24-06:00",
          "tree_id": "5cdc4c5902c76162c38005200716dc4590e34435",
          "url": "https://github.com/zcash/orchard/commit/7e90bf2a2c1c6d766496585a045ca110d6b8f1d8"
        },
        "date": 1776698062993,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2574988973,
            "range": "± 18205511",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2592143549,
            "range": "± 12206655",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 3709088949,
            "range": "± 34322788",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 4805877796,
            "range": "± 20982424",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 20272478,
            "range": "± 467699",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 20577306,
            "range": "± 169990",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 23858067,
            "range": "± 173158",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 27006372,
            "range": "± 1065576",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1478620,
            "range": "± 5070",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 124840,
            "range": "± 524",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1474608,
            "range": "± 5718",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1314498351,
            "range": "± 2920395",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 15619699,
            "range": "± 52705",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2112788,
            "range": "± 5688",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 15590815,
            "range": "± 43810",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2077101,
            "range": "± 2895",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 78066441,
            "range": "± 1923580",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10507757,
            "range": "± 29553",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 77886676,
            "range": "± 332621",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10331108,
            "range": "± 28813",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 156076336,
            "range": "± 263716",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21005631,
            "range": "± 34768",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 155806804,
            "range": "± 297477",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 20658389,
            "range": "± 35916",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 453466,
            "range": "± 3770",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 488513,
            "range": "± 1102",
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
          "id": "f119a4127203c8cd281afb791140ffbda9a7422a",
          "message": "Merge pull request #490 from zcash/detached_internal_versions\n\norchard_internal split clarifications",
          "timestamp": "2026-04-21T12:30:23-06:00",
          "tree_id": "808d4087649461fe00e01f07c37037658a4f867d",
          "url": "https://github.com/zcash/orchard/commit/f119a4127203c8cd281afb791140ffbda9a7422a"
        },
        "date": 1776796949351,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2593908180,
            "range": "± 46903546",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2590719569,
            "range": "± 12143283",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 3737586526,
            "range": "± 14659058",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 4872121050,
            "range": "± 43562969",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 20797036,
            "range": "± 172107",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 20792172,
            "range": "± 160416",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 23843444,
            "range": "± 219551",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 26999645,
            "range": "± 553551",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1487171,
            "range": "± 8463",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 124750,
            "range": "± 291",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1484594,
            "range": "± 13172",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1317240724,
            "range": "± 1770011",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 15686438,
            "range": "± 134124",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2111647,
            "range": "± 37015",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 15664466,
            "range": "± 78440",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2077854,
            "range": "± 10033",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 78406004,
            "range": "± 109856",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10497798,
            "range": "± 18188",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 78203624,
            "range": "± 186127",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10326448,
            "range": "± 19539",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 156908619,
            "range": "± 180718",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21036744,
            "range": "± 363792",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 156395508,
            "range": "± 216671",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 20679936,
            "range": "± 94340",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 452919,
            "range": "± 553",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 488303,
            "range": "± 4693",
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
          "id": "4a106e1ddd73cfcecf036283d6e725d417846a28",
          "message": "Merge pull request #492 from daira/alt/rk-identity-at-constructors\n\nconsensus: Reject identity `rk` in `Action::from_parts` and `Instance::from_parts`",
          "timestamp": "2026-04-22T12:30:49-06:00",
          "tree_id": "65fb80eae89ce34d491083c4d18b96b997e88ce1",
          "url": "https://github.com/zcash/orchard/commit/4a106e1ddd73cfcecf036283d6e725d417846a28"
        },
        "date": 1776883269665,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2135354376,
            "range": "± 16620168",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2112104836,
            "range": "± 4125897",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 3034972371,
            "range": "± 17681837",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 3970617933,
            "range": "± 29534394",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 17285387,
            "range": "± 246809",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 17434094,
            "range": "± 149143",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 19785394,
            "range": "± 205080",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 22525502,
            "range": "± 139131",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1227275,
            "range": "± 18120",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 104124,
            "range": "± 209",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1225083,
            "range": "± 2419",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1094003579,
            "range": "± 4739013",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 12966683,
            "range": "± 65744",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 1773340,
            "range": "± 6327",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 12942252,
            "range": "± 276658",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 1742042,
            "range": "± 14890",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 64775992,
            "range": "± 882886",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 8817369,
            "range": "± 9009",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 64651284,
            "range": "± 52046",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 8658002,
            "range": "± 25983",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 129603001,
            "range": "± 1199275",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 17629228,
            "range": "± 34892",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 129317401,
            "range": "± 1377281",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 17305152,
            "range": "± 75258",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 375628,
            "range": "± 1841",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 404887,
            "range": "± 518",
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
          "id": "081c2ac4fa0395f808a1d1e9b6d0c8e2744a53b5",
          "message": "Merge pull request #493 from ebfull/undo-orchard-internal\n\nRevert `orchard_internal` crate split",
          "timestamp": "2026-04-22T15:01:11-06:00",
          "tree_id": "404bdeafe2501ae80bfb427e0cfb3f5a9396a06e",
          "url": "https://github.com/zcash/orchard/commit/081c2ac4fa0395f808a1d1e9b6d0c8e2744a53b5"
        },
        "date": 1776892387262,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 2576805307,
            "range": "± 112254407",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 2568409766,
            "range": "± 9324508",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 3693670258,
            "range": "± 19370883",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 4819770461,
            "range": "± 30785670",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 20710205,
            "range": "± 298045",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 20793301,
            "range": "± 178846",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 23923819,
            "range": "± 183029",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 27025876,
            "range": "± 145962",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1485217,
            "range": "± 7000",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 124623,
            "range": "± 156",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1482306,
            "range": "± 10567",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 1332230804,
            "range": "± 1386510",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 15673193,
            "range": "± 196657",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 2115903,
            "range": "± 2766",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 15636401,
            "range": "± 56528",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2081135,
            "range": "± 31911",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 78272031,
            "range": "± 166873",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 10525287,
            "range": "± 15730",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 78069622,
            "range": "± 174195",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 10351405,
            "range": "± 24563",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 156434215,
            "range": "± 246143",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 21036897,
            "range": "± 645973",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 156089872,
            "range": "± 158136",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 20686140,
            "range": "± 19600",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 453172,
            "range": "± 6315",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 488516,
            "range": "± 4027",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}