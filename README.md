## Update Dns Record

### GCP

costs a few cents a month

```
/path/to/dyndns -v gcp --auth-file /path/to/service_account.json --project __project_name__  --zone __zone_name__ --hostname __hostname__
```

### Digital Ocean

Free

```
/path/to/udyndns -v digital-ocean --api-key-file /path/to/digital_ocean.token --hostname __hostname__
```

## Automation

Run Every 5 minutes in crontab

```
crontab -e
```

```
 */5 * * * * /path/to/udyndns  digital-ocean --api-key-file /path/to/digital_ocean.token --hostname __hostname__  >> /path/to/udyndns.logs    2>&1
```
