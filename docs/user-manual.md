# User Manual

## Introduction

pipelines-rs is a mainframe-style data pipeline processor that works with 80-byte fixed-width records. This manual covers how to use the Web UI and write pipeline specifications.

## User Guide

### Getting Started

1. **Build and run the application**:
   ```bash
   ./scripts/build.sh
   ./scripts/serve.sh
   ```

2. **Open the Web UI** at http://localhost:9952

3. The interface has three panels:
   - **Input Records** - Enter or paste 80-byte fixed-width records
   - **Pipeline** - Write your pipeline specification
   - **Output Records** - View the results after running

### Input Records

Records are fixed-width text lines, traditionally 80 bytes (matching punch card width). Each field occupies a specific column range:

```
0---------1---------2---------3---------4---------5---------6---------7---------
SMITH   JOHN      SALES     00050000
```

The ruler shows 0-based column positions:
- `0` marks position 0
- `1` marks position 10
- `2` marks position 20
- etc.

### Writing Pipelines

Pipelines follow this structure:

```
PIPE CONSOLE
| <stage>
| <stage>
| CONSOLE
?
```

- `PIPE CONSOLE` - Start the pipeline, reading from Input Records
- `| <stage>` - Apply a transformation stage
- `| CONSOLE` - Write results to Output Records
- `?` - End of pipeline

### Running a Pipeline

1. Enter your input records in the left panel
2. Write your pipeline in the center panel
3. Click **Run** to execute
4. View results in the right panel

### Loading and Saving Pipelines

- **Load** - Click to upload a `.pipe` file from your filesystem
- **Save** - Click to download the current pipeline as `pipeline.pipe`

Sample pipelines are available in the `specs/` directory.

---

## Reference

### Pipeline Structure

```
PIPE CONSOLE      # Required: Start pipeline with input source
| <stage>         # Zero or more transformation stages
| CONSOLE         # Required: End pipeline with output sink
?                 # Optional: Explicit end marker
```

### Comments

Lines starting with `#` are comments and ignored:

```
# This is a comment
PIPE CONSOLE
| FILTER 18,10 = "SALES"   # Inline comments not supported
| CONSOLE
?
```

### Stages (Alphabetical)

#### CHANGE

Replaces text in records (like sed).

**Syntax**:
```
CHANGE "old" "new"
```

**Parameters**:
- `old` - Text to find
- `new` - Replacement text (can be empty)

**Examples**:
```
CHANGE "SALES" "MKTG"       # Replace SALES with MKTG
CHANGE "ERROR: " ""         # Remove "ERROR: " prefix
```

#### CONSOLE

Reads from or writes to the console (Input/Output Records panels).

**Usage**:
- First stage: Reads records from Input Records panel
- Last stage: Writes records to Output Records panel

```
PIPE CONSOLE      # Read input
| ...
| CONSOLE         # Write output
?
```

#### COUNT

Counts records and outputs a single summary record.

**Syntax**:
```
COUNT
```

**Output**: A single record like `COUNT=42`

**Example**:
```
PIPE CONSOLE
| FILTER 18,10 = "SALES"
| COUNT
| CONSOLE
?
```
Outputs: `COUNT=3` (if 3 SALES records)

#### DUPLICATE

Repeats each record n times.

**Syntax**:
```
DUPLICATE n
```

**Parameter**:
- `n` - Number of copies (must be >= 1)

**Example**:
```
DUPLICATE 2                 # Each record appears twice
```

#### FILTER

Keeps or removes records based on field comparison.

**Syntax**:
```
FILTER pos,len = "value"    # Keep records where field equals value
FILTER pos,len != "value"   # Keep records where field does NOT equal value
```

**Parameters**:
- `pos` - Starting column position (0-based)
- `len` - Field length in characters
- `value` - String to compare (must be quoted)

**Examples**:
```
FILTER 18,10 = "SALES"      # Keep records with "SALES" at columns 18-27
FILTER 0,8 != "SMITH"       # Remove records with "SMITH" at columns 0-7
```

#### LITERAL

Appends a literal text record to the stream.

**Syntax**:
```
LITERAL "text"
```

**Parameter**:
- `text` - The literal text to add as a record

**Example**:
```
LITERAL "--- END OF REPORT ---"   # Add footer record
```

#### LOCATE

Keeps records containing a pattern (like grep).

**Syntax**:
```
LOCATE "pattern"              # Search entire record
LOCATE pos,len "pattern"      # Search specific field only
```

**Parameters**:
- `pattern` - Text to search for
- `pos,len` - Optional field to restrict search

**Examples**:
```
LOCATE "ERROR"                # Keep records containing ERROR
LOCATE 18,10 "SALES"          # Keep if field at 18,10 contains SALES
```

#### LOWER

Converts all records to lowercase.

**Syntax**:
```
LOWER
```

**Example**:
```
LOWER                         # "SMITH" becomes "smith"
```

#### NLOCATE

Keeps records NOT containing a pattern (inverse of LOCATE).

**Syntax**:
```
NLOCATE "pattern"             # Search entire record
NLOCATE pos,len "pattern"     # Search specific field only
```

**Examples**:
```
NLOCATE "ERROR"               # Keep records without ERROR
NLOCATE 18,10 "SALES"         # Keep if field doesn't contain SALES
```

#### REVERSE

Reverses characters in each record.

**Syntax**:
```
REVERSE
```

**Example**:
```
REVERSE                       # "Hello" becomes "olleH"
```

#### SELECT

Extracts and repositions fields to create new records.

**Syntax**:
```
SELECT src,len,dest; src,len,dest; ...
```

**Parameters** (for each field):
- `src` - Source column position (0-based)
- `len` - Field length to copy
- `dest` - Destination column position in output record

**Example**:
```
SELECT 0,8,0; 28,8,8        # Copy columns 0-7 to 0-7, columns 28-35 to 8-15
```

This transforms:
```
SMITH   JOHN      SALES     00050000
```
Into:
```
SMITH   00050000
```

#### SKIP

Skips the first N records, keeping the rest.

**Syntax**:
```
SKIP n
```

**Parameter**:
- `n` - Number of records to skip

**Example**:
```
SKIP 3                      # Skip first 3 records, keep the rest
```

#### TAKE

Keeps only the first N records.

**Syntax**:
```
TAKE n
```

**Parameter**:
- `n` - Number of records to keep

**Example**:
```
TAKE 5                      # Keep first 5 records
```

#### UPPER

Converts all records to uppercase.

**Syntax**:
```
UPPER
```

**Example**:
```
UPPER                       # "Smith" becomes "SMITH"
```

---

## Examples

### filter-sales.pipe

Filter records for the SALES department only.

```pipe
# Filter records for SALES department
# Input: Employee records (Last, First, Dept, Salary)
PIPE CONSOLE
| FILTER 18,10 = "SALES"
| CONSOLE
?
```

**What it does**: Reads all input records and outputs only those where columns 18-27 contain "SALES" (padded with spaces to 10 characters).

**Input**:
```
SMITH   JOHN      SALES     00050000
JONES   MARY      ENGINEER  00075000
DOE     JANE      SALES     00060000
```

**Output**:
```
SMITH   JOHN      SALES     00050000
DOE     JANE      SALES     00060000
```

---

### sales-report.pipe

Create a report showing SALES employees with just their name and salary.

```pipe
# Sales department report
# Filter SALES and extract name + salary
PIPE CONSOLE
| FILTER 18,10 = "SALES"
| SELECT 0,8,0; 28,8,8
| CONSOLE
?
```

**What it does**:
1. Filters for SALES department records
2. Extracts last name (columns 0-7) and salary (columns 28-35)
3. Repositions them into a compact output format

**Input**:
```
SMITH   JOHN      SALES     00050000
JONES   MARY      ENGINEER  00075000
DOE     JANE      SALES     00060000
```

**Output**:
```
SMITH   00050000
DOE     00060000
```

---

### engineers-only.pipe

Filter for engineering staff.

```pipe
# Filter for engineering staff only
PIPE CONSOLE
| FILTER 18,10 = "ENGINEER"
| CONSOLE
?
```

**What it does**: Outputs only records where the department field (columns 18-27) is "ENGINEER".

---

### top-five.pipe

Get the first 5 records with selected fields.

```pipe
# Get first 5 records with name and department
PIPE CONSOLE
| SELECT 0,8,0; 8,10,8; 18,10,18
| TAKE 5
| CONSOLE
?
```

**What it does**:
1. Selects last name, first name, and department fields
2. Keeps only the first 5 records

**Field mapping**:
- Columns 0-7 (last name) → Output columns 0-7
- Columns 8-17 (first name) → Output columns 8-17
- Columns 18-27 (department) → Output columns 18-27

---

### non-marketing.pipe

Exclude marketing department records.

```pipe
# Exclude marketing department
# Shows all employees except MARKETING
PIPE CONSOLE
| FILTER 18,10 != "MARKETING"
| CONSOLE
?
```

**What it does**: Uses the `!=` operator to exclude (rather than include) records matching a condition. Outputs all records EXCEPT those in MARKETING.

---

## Tips

### Column Counting

Use the ruler above the input area to identify column positions:
```
0---------1---------2---------3---------4---------5---------6---------7---------
```
- Position 0 is the first character
- Position 10 is where "1" appears
- Position 20 is where "2" appears

### Field Padding

When filtering, remember that fields are space-padded:
- `"SALES"` in a 10-character field is actually `"SALES     "` (with 5 trailing spaces)
- The FILTER stage handles this automatically for the comparison

### Chaining Stages

Stages execute in order. Data flows through each stage:
```
PIPE CONSOLE
| FILTER 18,10 = "SALES"    # First: filter to SALES only
| SELECT 0,8,0; 28,8,8      # Then: extract fields
| TAKE 10                   # Finally: limit to 10 records
| CONSOLE
?
```

---

## Related Documentation

- [Architecture](architecture.md) - System design overview
- [Design](design.md) - Technical design decisions
- [Status](status.md) - Current project status
