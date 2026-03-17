#!/bin/bash
# Log rotation script for bills_to_db application
# Keeps logs for 90 days, compresses old logs, runs daily via cron

LOG_DIR="logs"
APP_LOG="$LOG_DIR/app.log"
ARCHIVE_DIR="$LOG_DIR/archive"

# Create archive directory if it doesn't exist
mkdir -p "$ARCHIVE_DIR"

# Rotate current log if it exists and is non-empty
if [ -f "$APP_LOG" ] && [ -s "$APP_LOG" ]; then
    # Create timestamp for archive file
    TIMESTAMP=$(date +%Y%m%d_%H%M%S)
    ARCHIVE_FILE="$ARCHIVE_DIR/app_$TIMESTAMP.log.gz"
    
    # Compress and archive current log
    gzip -c "$APP_LOG" > "$ARCHIVE_FILE"
    
    # Clear the current log file
    > "$APP_LOG"
    
    echo "Rotated log to $ARCHIVE_FILE"
fi

# Delete archives older than 90 days
find "$ARCHIVE_DIR" -name "app_*.log.gz" -type f -mtime +90 -delete

echo "Log rotation completed"