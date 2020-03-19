# single-use-dns

Simple throwaway dns server that temporarily hosts records for a single domain name.

Useful to temporarily host a DNS record, such as a ACME DNS-01 challenge.

## Example

```
$ single-use-dns --domain _acme-challenge.example.com --txt O_FFiiKTKtSYllnIKhXteCYji_d2vDk_FFiiKTKtSYl
Listening on [::]:53 (UDP and TCP)
Serving 1 record(s) for _acme-challenge.example.com
```
