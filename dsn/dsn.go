package dsn

import (
    "fmt"
    "regexp"
    "errors"
)

type Dsn struct {
    raw     string
    Scheme  string
    Id      string
    Secret  string
    Host    string
    Port    string
    Path    string
}

func extractParts(rawdsn string) ([]string, error) {
    re := regexp.MustCompile(NamedExpression("scheme", SCHEME_REGEXP) +
        "?://" +
        NamedExpression("id", ID_REGEXP) +
        "?:" +
        NamedExpression("secret", SECRET_REGEXP) +
        "?@" +
        NamedExpression("host", OrExpressions(BUCKET_REGEXP, HOST_REGEXP)) +
        "?:" +
        NamedExpression("port", OrExpressions(PORT_REGEXP, REGION_REGEXP)) +
        "?/" +
        NamedExpression("path", PATH_REGEXP))

    parts := re.FindStringSubmatch(rawdsn)
    if len(parts) == 0 {
        return nil, errors.New(fmt.Sprintf("No dsn mathched in %s", rawdsn))
    }

    return parts[1:], nil
}

func Parse(rawdsn string) (dsn *Dsn, err error) {
    parts, err := extractParts(rawdsn)
    if err != nil {
        return nil, err
    }

    // Init Dsn instance with url parsed informations
    // we can thrust
    dsn = &Dsn{
        raw: rawdsn,
        Scheme: parts[0],
        Id: parts[1],
        Secret: parts[2],
        Host: parts[3],
        Port: parts[4],
        Path: parts[5],
    }

    return dsn, nil
}
