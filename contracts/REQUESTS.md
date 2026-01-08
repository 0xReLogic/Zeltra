# API Requests

Frontend → Backend communication channel.

---

## How to Use

### Frontend:
1. Tulis request baru di "Active Requests"
2. Pakai format template
3. Set status 🟡 Pending

### Backend:
1. Review request
2. Update status:
   - 🟢 Accepted (will implement)
   - 🔴 Rejected (with reason)
   - ✅ Done (implemented, update openapi.yaml)
3. Add response/notes

---

## Template

```markdown
### [REQ-XXX] Title
**Status:** 🟡 Pending
**Priority:** High / Medium / Low
**Date:** YYYY-MM-DD

**Need:**
Apa yang dibutuhkan dan kenapa.

**Proposed Endpoint:**
`METHOD /path`

**Request Body:**
\`\`\`json
{}
\`\`\`

**Expected Response:**
\`\`\`json
{}
\`\`\`

**Backend Response:**
> (Backend isi di sini)
```

---

## Active Requests

(Kosong - belum ada request)

---

## Accepted (In Progress)

(Kosong)

---

## Completed

(Kosong)

---

## Rejected

(Kosong)
