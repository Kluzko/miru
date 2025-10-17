#!/bin/bash
# Cleanup script for leftover test databases
# Run this if test databases weren't cleaned up due to panics or interruptions

set -e

# Load environment variables
if [ -f "../../.env" ]; then
    export $(cat ../../.env | grep -v '^#' | xargs)
fi

DB_URL="${TEST_DATABASE_URL:-$DATABASE_URL}"

echo "Cleaning up test databases..."

# Get list of test databases
TEST_DBS=$(psql "$DB_URL" -t -c "SELECT datname FROM pg_database WHERE datname LIKE 'test_db_%';")

if [ -z "$TEST_DBS" ]; then
    echo "✓ No test databases to clean up"
    exit 0
fi

COUNT=0
for db in $TEST_DBS; do
    echo "Dropping $db..."

    # Terminate connections
    psql "$DB_URL" -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '$db';" > /dev/null 2>&1 || true

    # Drop database
    psql "$DB_URL" -c "DROP DATABASE IF EXISTS $db;" > /dev/null 2>&1 || echo "  ⚠️  Failed to drop $db"

    COUNT=$((COUNT + 1))
done

echo "✓ Cleaned up $COUNT test databases"
