#!/bin/bash

# File: scripts/peer-health.sh
# Connectivity and latency check on siblings from siblings.json using curl and date

SIBLINGS_FILE="siblings.json"

if [ ! -f "$SIBLINGS_FILE" ]; then
    printf '{"error": "%s not found"}\n' "$SIBLINGS_FILE"
    exit 1
fi

# Get the count and the list of siblings
SIBLINGS_DATA=$(cat "$SIBLINGS_FILE")
COUNT=$(echo "$SIBLINGS_DATA" | jq -r '.count // 0')

if [ "$COUNT" -eq 0 ]; then
    printf '{"results": [], "count": 0}\n'
    exit 0
fi

RESULTS="[]"

# Get sibling urls
SIBLING_URLS=$(echo "$SIBLINGS_DATA" | jq -r '.siblings[].url')

for URL in $SIBLING_URLS; do
    if [ "$URL" = "null" ] || [ -z "$URL" ]; then
        continue
    fi

    # Record start time in nanoseconds
    START_NS=$(date +%s%N)
    
    # Perform health check on /health endpoint
    # Timeout 5s, silence output, just get status code
    HTTP_CODE=$(curl -s -L -o /dev/null -w "%{http_code}" "${URL%/}/health" --max-time 5 || echo "0")
    
    # Record end time in nanoseconds
    END_NS=$(date +%s%N)
    
    # Calculate latency in ms (approx)
    if echo "$START_NS" | grep -qE '^[0-9]+$' && echo "$END_NS" | grep -qE '^[0-9]+$'; then
        LATENCY_NS=$((END_NS - START_NS))
        LATENCY_MS=$((LATENCY_NS / 1000000))
    else
        LATENCY_MS=-1
    fi
    
    HEALTHY="false"
    if [ "$HTTP_CODE" = "200" ]; then
        HEALTHY="true"
    fi
    
    # Ensure HTTP_CODE is a number
    case "$HTTP_CODE" in
        ''|*[!0-9]*) HTTP_CODE=0 ;;
    esac

    RESULT=$(jq -n \
        --arg url "$URL" \
        --argjson code "$HTTP_CODE" \
        --argjson latency "$LATENCY_MS" \
        --argjson healthy "$HEALTHY" \
        '{url: $url, status_code: $code, latency_ms: $latency, healthy: $healthy}')
    
    # Add to results array
    RESULTS=$(echo "$RESULTS" | jq --argjson res "$RESULT" '. += [$res]')
done

# Output final JSON object
echo "$RESULTS" | jq --arg count "$(echo "$RESULTS" | jq '. | length')" '{results: ., count: ($count | tonumber)}'
