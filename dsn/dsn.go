package dsn

import (
    "regexp"
    "strconv"
)

type Dsn struct {
    raw     string
    Scheme  string
    Id      string
    Secret  string
    Host    string
    Port    int
    Path    string
}

func extractParts(rawdsn string) []string {
    re := regexp.MustCompile("(?P<scheme>[a-zA-Z0-9]+)" +
        "?://" +
        "(?P<id>[^:]+)" +
        "?:" +
        "(?P<secret>[^@]+)" +
        "?@" +
        "(?P<host>[a-zA-Z0-9]+)" +
        "?:" +
        "(?P<port>[0-9]+)" +
        "?/" +
        "(?P<path>[a-zA-Z0-9/]+)")
    return re.FindStringSubmatch(rawdsn)[1:]
}

func Parse(rawdsn string) (dsn *Dsn, err error) {
    parts := extractParts(rawdsn)

    // Init Dsn instance with url parsed informations
    // we can thrust
    dsn = &Dsn{
        raw: rawdsn,
        Scheme: parts[0],
        Id: parts[1],
        Secret: parts[2],
        Host: parts[3],
        Path: parts[5],
    }

    // Extract host and port from the parsed URL
    // instance Host string
    if port := parts[4]; port != "" {
        dsn.Port, err = strconv.Atoi(port)
        if err != nil {
            return nil, err
        }
    }

    return dsn, nil
}
