# Update EC2 DNS

This is a small program that will update the DNS entry for an EC2 instance according to its state.


## Development

This is built using up-to-date stable Rust. Older versions are not supported.

To run locally:

Start the backend:

```
cargo lambda watch
```

Send a payload:

```
cargo lambda invoke --data-ascii \
'{ "version": "0",
  "id": "6a7e8feb-b491-4cf7-a9f1-bf3703467718",
  "detail-type": "EC2 Instance State-change Notification",
  "source": "aws.ec2",
  "account": "111122223333",
  "time": "2017-12-22T18:43:48Z",
  "region": "us-west-1",
  "resources": [
    "arn:aws:ec2:us-west-1:123456789012:instance/i-1234567890abcdef0"
  ],
  "detail": {
    "instance-id": "i-1234567890abcdef0",
    "state": "terminated"
  }
}'
```

## License

This program is distributed under the terms of the [3-Clause BSD License](LICENSE).
