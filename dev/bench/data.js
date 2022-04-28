window.BENCHMARK_DATA = {
  "lastUpdate": 1651177487977,
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
          "id": "62ca755e389b6f78c4432ed3806464fecc2b3e2b",
          "message": "Merge pull request #251 from zcash/update-halo2-constraints-helper\n\nMigrate to `halo2::plonk::Constraints` helper",
          "timestamp": "2022-04-28T21:12:28+01:00",
          "tree_id": "b9ea46220cec0167f569611642d89a763900313b",
          "url": "https://github.com/zcash/orchard/commit/62ca755e389b6f78c4432ed3806464fecc2b3e2b"
        },
        "date": 1651177486346,
        "tool": "cargo",
        "benches": [
          {
            "name": "proving/bundle/1",
            "value": 4194067121,
            "range": "± 27843650",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/2",
            "value": 4178251157,
            "range": "± 10899685",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/3",
            "value": 5987118594,
            "range": "± 21672197",
            "unit": "ns/iter"
          },
          {
            "name": "proving/bundle/4",
            "value": 7788350520,
            "range": "± 198156943",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/1",
            "value": 33195486,
            "range": "± 177269",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/2",
            "value": 33082785,
            "range": "± 326157",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/3",
            "value": 36821962,
            "range": "± 189512",
            "unit": "ns/iter"
          },
          {
            "name": "verifying/bundle/4",
            "value": 40560039,
            "range": "± 268938",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}