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

````markdown
### [REQ-XXX] Title

**Status:** 🟡 Pending
**Priority:** High / Medium / Low
**Date:** YYYY-MM-DD

**Need:**
Apa yang dibutuhkan dan kenapa.

**Proposed Endpoint:**
`METHOD /path`

**Request Body:**

```json
{}
```
````

**Expected Response:**

```json
{}
```

**Backend Response:**

> (Backend isi di sini)

````

---

## Active Requests

### [REQ-001] Organization & Team Management APIs
**Status:** 🟡 Pending
**Priority:** High
**Date:** 2026-01-08

**Need:**
UI screens for Organization Settings (Currency, Timezone) and Team Management (Invite, Role, Remove).

**Proposed Endpoints:**

1. **Get Organization Details**
   `GET /organizations/:id`

2. **Update Organization**
   `PATCH /organizations/:id`
   ```json
   { "base_currency": "USD", "timezone": "Asia/Jakarta" }
````

3. **List Organization Users**
   `GET /organizations/:id/users`

4. **Invite User**
   `POST /organizations/:id/users`

   ```json
   { "email": "user@example.com", "role": "accountant" }
   ```

5. **Update User Role**
   `PATCH /organizations/:id/users/:userId`

   ```json
   { "role": "admin" }
   ```

6. **Remove User**
   `DELETE /organizations/:id/users/:userId`

**Expected Response:**
Standard JSON responses as per API patterns.

---

## Accepted (In Progress)

(Kosong)

---

## Completed

(Kosong)

---

## Rejected

(Kosong)
