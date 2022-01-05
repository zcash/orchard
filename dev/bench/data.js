window.BENCHMARK_DATA = {
  "lastUpdate": 1641387343489,
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
          "id": "5742eb5c52687e3f3316072c885a093f75fd9a92",
          "message": "Merge pull request #269 from zcash/pin-dependencies\n\nPin `pprof = 0.6.1`.",
          "timestamp": "2022-01-05T12:27:32Z",
          "tree_id": "272641e88184ac22ea08c04c60894552fac37cf6",
          "url": "https://github.com/zcash/orchard/commit/5742eb5c52687e3f3316072c885a093f75fd9a92"
        },
        "date": 1641387342558,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 7151508601,
            "range": "± 179171882",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 7280072981,
            "range": "± 184324507",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 10809110901,
            "range": "± 346883447",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 14212045408,
            "range": "± 558360089",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 39293920,
            "range": "± 1691004",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 39544054,
            "range": "± 1362034",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 44194200,
            "range": "± 1942855",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 46925625,
            "range": "± 2760988",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/valid",
            "value": 1395589,
            "range": "± 29446",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/invalid",
            "value": 178468,
            "range": "± 3757",
            "unit": "ns/iter"
          },
          {
            "name": "note-decryption/compact-valid",
            "value": 1200599,
            "range": "± 46501",
            "unit": "ns/iter"
          },
          {
            "name": "compact-note-decryption/invalid",
            "value": 154040586,
            "range": "± 6272313",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/10",
            "value": 27488179,
            "range": "± 466215",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/10",
            "value": 3050017,
            "range": "± 89525",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/10",
            "value": 22498196,
            "range": "± 1046471",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/10",
            "value": 2568644,
            "range": "± 134499",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/50",
            "value": 135346757,
            "range": "± 3053284",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/50",
            "value": 13698350,
            "range": "± 545791",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/50",
            "value": 118530395,
            "range": "± 5726260",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/50",
            "value": 14223173,
            "range": "± 672322",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/valid/100",
            "value": 276120892,
            "range": "± 5153980",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/invalid/100",
            "value": 28140901,
            "range": "± 1892483",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-valid/100",
            "value": 244996596,
            "range": "± 9419965",
            "unit": "ns/iter"
          },
          {
            "name": "batch-note-decryption/compact-invalid/100",
            "value": 27014900,
            "range": "± 1342849",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-prover",
            "value": 72714428,
            "range": "± 16073192",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 3, RATE = 2-verifier",
            "value": 4145416,
            "range": "± 166031",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-prover",
            "value": 169555952,
            "range": "± 5791694",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 9, RATE = 8-verifier",
            "value": 6097903,
            "range": "± 253930",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-prover",
            "value": 245824579,
            "range": "± 8009583",
            "unit": "ns/iter"
          },
          {
            "name": "WIDTH = 12, RATE = 11-verifier",
            "value": 7091823,
            "range": "± 314148",
            "unit": "ns/iter"
          },
          {
            "name": "Poseidon/2-to-1",
            "value": 44855,
            "range": "± 1937",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/510",
            "value": 171121,
            "range": "± 6637",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/510",
            "value": 174083,
            "range": "± 9051",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/510",
            "value": 291108,
            "range": "± 17268",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/510",
            "value": 282674,
            "range": "± 16438",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/520",
            "value": 155916,
            "range": "± 8350",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/520",
            "value": 170167,
            "range": "± 10054",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/520",
            "value": 299765,
            "range": "± 18063",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/520",
            "value": 295988,
            "range": "± 17786",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash-to-point/1086",
            "value": 364649,
            "range": "± 22048",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/hash/1086",
            "value": 354560,
            "range": "± 25767",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/commit/1086",
            "value": 497279,
            "range": "± 21669",
            "unit": "ns/iter"
          },
          {
            "name": "Sinsemilla/short-commit/1086",
            "value": 479544,
            "range": "± 20599",
            "unit": "ns/iter"
          },
          {
            "name": "derive_fvk",
            "value": 601975,
            "range": "± 34433",
            "unit": "ns/iter"
          },
          {
            "name": "default_address",
            "value": 704427,
            "range": "± 38520",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}