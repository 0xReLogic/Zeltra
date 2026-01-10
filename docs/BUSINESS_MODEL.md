# Business Model

Zeltra: Enterprise-grade B2B Expense & Budgeting Engine

---

## Positioning

> "Enterprise accounting at Xero prices"

Zeltra fills the gap between:
- **Basic tools** (Xero, QuickBooks) - flat pricing but limited features
- **Enterprise ERP** (Sage Intacct, NetSuite) - powerful but expensive

---

## Competitive Landscape (Real Data - Jan 2026)

### Expense Management Players

| Product | Pricing | Core Focus | Weakness |
|---------|---------|------------|----------|
| **Expensify** | $5-36/user/mo | Receipt scanning, expense tracking, corporate cards | No accounting, expense-only solution |
| **Zoho Expense** | $4-7/user/mo | Budget-friendly expense management | Limited to expense workflows |
| **Ramp** | Free - $15/user/mo | Corporate cards, spend control | Card-centric, no ledger |
| **Brex** | Free + quote | Corporate cards, startups | Card-centric, no simulation |
| **SAP Concur** | $9-50+/user/mo (quote) | Enterprise travel + expense | Expensive, legacy UI, complex |

### Accounting/ERP Players

| Product | Pricing | Core Focus | Weakness |
|---------|---------|------------|----------|
| **Xero** | $65/month flat | SME accounting, easy to use | Limited dimensions, no simulation |
| **QuickBooks** | $30-200/mo | SMB accounting | Not enterprise-ready |
| **Sage Intacct** | ~$400/mo + $99/user | Dimensional accounting, multi-entity | Expensive for mid-market |
| **NetSuite** | $999/mo + $99/user | Full ERP | Overkill, expensive |

### Feature Comparison

| Feature | Expensify | Zoho Expense | Xero | QuickBooks | Sage Intacct | **Zeltra** |
|---------|-----------|--------------|------|------------|--------------|------------|
| Expense tracking | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Receipt OCR | ✅ | ✅ | ❌ | ❌ | ✅ | ❌ (roadmap) |
| Corporate cards | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Mobile app | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ (roadmap) |
| Multi-currency (proper) | ❌ | ❌ | ❌ | Limited | ✅ | ✅ 3-value |
| Dimensional accounting | ❌ | ❌ | ❌ | ❌ | ✅ | ✅ |
| Budget simulation | ❌ | ❌ | ❌ | ❌ | Limited | ✅ Real-time |
| Double-entry ledger | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ |
| On-premise option | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |
| Approval workflow | ✅ | ✅ | Basic | ✅ | ✅ | ✅ |
| Financial reports | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ |
| Self-hosted | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ |

---

## Zeltra's Real Differentiators

### 1. Budget Simulation Engine (UNIQUE)
- Real-time "what-if" scenarios
- "If revenue drops 20%, how long is our runway?"
- "If we hire 10 people, what's the budget impact?"
- Parallel processing (Rust + Rayon) for instant results
- **No competitor at mid-market price has this**

### 2. Multi-Currency 3-Value Storage
- Store: source_amount + exchange_rate + functional_amount
- Proper currency revaluation at period-end
- Full audit trail of historical rates
- **Enterprise-grade, not convert-on-the-fly**

### 3. Dimensional Accounting at Mid-Market Price
- Sage Intacct charges $400/mo + $99/user for this
- Zeltra: included in Growth tier ($25/month flat)
- Slice data by department, project, cost center, location
- **Same power, 94% cheaper**

### 4. Self-Hosted / On-Premise Option
- Ramp, Brex, Expensify: SaaS only
- Banks, healthcare, government need data sovereignty
- Modern Rust stack, not legacy Java
- **Niche but high-value segment**

### 5. Performance (Technical)
- Rust backend = fast, memory-safe, no GC pauses
- Handle high transaction volumes
- Lower infrastructure costs
- **Matters for high-volume customers**

---

## What Zeltra Does NOT Have (Honest)

- ❌ Corporate cards (Ramp/Brex core feature - $0 financing)
- ❌ Receipt OCR (Expensify/Zoho core feature - SmartScan)
- ❌ Mobile app (All competitors have mature mobile apps)
- ❌ Integrations ecosystem (QuickBooks, Slack, 800+ apps)
- ❌ Brand recognition / trust (15M users for Expensify)
- ❌ Customer reviews / case studies (Established competitors have thousands)
- ❌ Instant reimbursement (Expensify/Ramp feature)
- ❌ Policy enforcement AI (Expensify SmartScanning AI)

---

## Target Market

### Tier 1: Growth Companies (Primary)

Profile:
- 50-500 employees
- Series A/B funded startups
- Digital agencies, software houses
- Multi-department, need budget visibility

Why Zeltra:
- Outgrown Expensify/Zoho Expense (expense-only solutions)
- Can't afford Sage Intacct ($15k+/year)
- Need dimensional reporting
- Want simulation for planning

### Tier 2: Expense-to-Accounting Transition (Secondary)

Profile:
- 20-200 employees
- Currently using Expensify/Zoho + QuickBooks/Xero
- Manual reconciliation between expense and accounting
- Need unified system

Why Zeltra:
- Single platform for expenses + accounting
- Eliminates manual data entry between systems
- 81% cheaper than Xero + Expensify combo
- Advanced features not available in expense tools

### Tier 3: Mid-Market Multi-Currency

Profile:
- 200-1000 employees
- Multi-country operations
- Multiple currencies daily
- Compliance requirements

Why Zeltra:
- Proper multi-currency (not convert-on-fly)
- Currency revaluation for month-end
- Audit trail for compliance

### Tier 3: Regulated Industries (On-Premise)

Profile:
- Banks, fintech, healthcare
- Government contractors
- Strict data sovereignty
- Cannot use cloud for financial data

Why Zeltra:
- Self-hosted option
- Modern stack (not legacy)
- Full control over data

---

## Pricing Strategy

### Cloud SaaS Pricing (Tiered Flat Monthly)

| Tier | Price | User Limits | Target | Key Features |
|------|-------|-------------|--------|--------------|
| **Starter** | $12/month | Up to 50 users | Small teams (5-50) | Basic expense, single currency, 2 dimensions |
| **Growth** | $25/month | Up to 200 users | Mid-market (50-200) | Multi-currency, unlimited dimensions, budgets |
| **Enterprise** | $45/month | Unlimited users | Large (200+) | Simulation, API, SSO, dedicated support |

**Effective Per-User Cost:**
- Starter: $0.24/user (for 50 users)
- Growth: $0.125/user (for 200 users)
- Enterprise: Varies by team size

Comparison:
- Starter ($12) vs Xero Established ($65) - 81% cheaper, +dimensions
- Growth ($25) vs QuickBooks Plus ($90) - 72% cheaper, +multi-currency
- Enterprise ($45) vs Sage Intacct ($400+/mo) - 89% cheaper, similar features

### Self-Hosted License

> **Pricing Strategy:** Murah dulu untuk build trust & case studies. Naikin harga setelah punya 3-5 reference customers.

| Model | Price | Includes |
|-------|-------|----------|
| **Annual License** | $5,000/year | Full features, updates, email support |
| **Perpetual License** | $15,000 one-time | + $3,000/year maintenance (optional) |
| **Enterprise Plus** | $20,000/year | Multi-entity, 20hrs custom dev, priority support |

Professional Services:
- Implementation support: $2,000 - $5,000
- Custom integration: $100/hour
- Training: $500/day (remote)

**Future Pricing (after 5+ customers):**
- Annual: $8,000-12,000/year
- Perpetual: $25,000-35,000
- Enterprise: $40,000/year

---

## Self-Hosted Deep Dive

### What They Get

Self-hosted = **full stack deployment** di server client:

| Component | Included |
|-----------|----------|
| Rust Backend | API server + Ledger Core engine |
| Next.js Frontend | Complete dashboard UI |
| Database | PostgreSQL schema + migrations |
| Infrastructure | Docker Compose + Kubernetes configs |
| Documentation | Deployment guide, API docs |
| License Key | Cryptographically signed validation |

Mereka deploy dan manage sendiri. Data 100% di server mereka.

### Why Companies Want Self-Hosted

1. **Data Sovereignty** - Bank, healthcare, government GAK BOLEH data finance di cloud vendor
2. **Compliance** - GDPR strict, data residency laws, industry regulations
3. **Security** - Air-gapped networks, internal security policies
4. **Control** - Custom integrations, internal system connections
5. **Performance** - High-volume processing tanpa latency ke cloud

### Target Customers

- Regional banks & credit unions
- Healthcare organizations (HIPAA)
- Government contractors
- Large enterprises with strict IT policies
- Companies in countries with data localization laws

### License Models Explained

**Annual License ($5,000/year)**
```
- Full access to latest version
- All updates & patches during subscription
- Email support (48hr response)
- Stop paying = keep current version, no updates
- Good for: Companies wanting latest features, lower commitment
```

**Perpetual License ($15,000 one-time)**
```
- Own the version forever
- Optional maintenance: $3,000/year (20%)
  - With maintenance: updates + support
  - Without: stuck on purchased version
- Good for: Budget certainty, long-term planning
```

**Enterprise Plus ($20,000/year)**
```
- Everything in Annual
- Multi-entity support (multiple orgs)
- 20 hours custom development/year
- Priority support (24hr response)
- Quarterly review calls
- Good for: Larger companies needing customization
```

### Sales Process (Manual)

```
┌─────────────────────────────────────────────────────────┐
│  SELF-HOSTED SALES FLOW                                 │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  1. INQUIRY                                             │
│     └─ Client contacts via website/email                │
│     └─ Initial call to understand requirements          │
│                                                         │
│  2. EVALUATION                                          │
│     └─ Provide demo environment (time-limited)          │
│     └─ Technical deep-dive with their IT team           │
│     └─ Security questionnaire / compliance review       │
│                                                         │
│  3. PROPOSAL                                            │
│     └─ Custom quote based on:                           │
│        - License model (annual/perpetual)               │
│        - Professional services needed                   │
│        - Support level required                         │
│                                                         │
│  4. CONTRACT                                            │
│     └─ Legal review (their side)                        │
│     └─ Sign license agreement                           │
│     └─ Payment (wire transfer / invoice)                │
│                                                         │
│  5. DELIVERY                                            │
│     └─ Generate license key                             │
│     └─ Grant access to:                                 │
│        - Private Docker registry, OR                    │
│        - Private GitHub repo                            │
│     └─ Provide deployment documentation                 │
│                                                         │
│  6. IMPLEMENTATION                                      │
│     └─ Client deploys (self-serve or with our help)     │
│     └─ Optional: paid implementation support            │
│     └─ Go-live validation                               │
│                                                         │
│  7. ONGOING                                             │
│     └─ Support tickets (based on SLA)                   │
│     └─ Updates via registry/repo (if subscribed)        │
│     └─ Annual renewal (for annual/maintenance)          │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### License Key System

**Approach: Offline License File (Recommended)**

```
┌─────────────────────────────────────────────────────────┐
│  LICENSE VALIDATION                                     │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  License File (JSON + Signature):                       │
│  {                                                      │
│    "license_id": "lic_abc123",                          │
│    "customer": "Acme Bank",                             │
│    "type": "perpetual",                                 │
│    "issued_at": "2026-03-15",                           │
│    "expires_at": null,  // null = perpetual             │
│    "maintenance_until": "2027-03-15",                   │
│    "features": ["all"],                                 │
│    "max_users": null,   // null = unlimited             │
│    "signature": "base64_ed25519_signature..."           │
│  }                                                      │
│                                                         │
│  Validation Flow:                                       │
│  1. App reads license.json on startup                   │
│  2. Verify signature with embedded public key           │
│  3. Check expiry (if applicable)                        │
│  4. Enable features based on license                    │
│  5. No internet required (air-gap friendly)             │
│                                                         │
│  Tampering Protection:                                  │
│  - Ed25519 signature (can't forge without private key)  │
│  - Public key embedded in binary                        │
│  - License tied to customer name (visible in UI)        │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

**Why NOT Phone-Home Licensing:**
- Enterprise clients often have air-gapped networks
- They hate software that "calls home"
- Adds failure point (what if our server down?)
- Trust issue - they're paying $75k, don't treat them like pirates

### Delivery Options

**Option A: Private Docker Registry**
```bash
# Client adds our registry credentials
docker login registry.zeltra.io

# Pull images
docker pull registry.zeltra.io/zeltra/api:v1.2.0
docker pull registry.zeltra.io/zeltra/web:v1.2.0

# Deploy with their docker-compose/k8s
```

**Option B: Private GitHub Repo**
```bash
# Client gets read access to private repo
git clone git@github.com:zeltra-enterprise/zeltra.git

# Build themselves (for customization)
cargo build --release
npm run build
```

Most enterprise clients prefer Docker (easier), some want source (audit/customize).

### Support Tiers

| Level | Response Time | Channels | Included In |
|-------|---------------|----------|-------------|
| Standard | 48 hours | Email | Annual License |
| Priority | 24 hours | Email + Slack | Enterprise Plus |
| Critical | 4 hours | Phone + Slack | Enterprise Plus (P1 issues) |

### Renewal & Churn Prevention

- 90 days before expiry: renewal reminder
- 60 days: renewal quote sent
- 30 days: escalate to account manager
- On expiry (annual): access to updates stops, app keeps running
- On expiry (maintenance): same as annual

---

## Revenue Model

### SaaS Revenue

1. **Subscription (MRR)** - Primary driver
2. **Overage fees** - Extra users/transactions beyond tier
3. **Add-ons** (future):
   - Receipt OCR: +$3/user/mo (vs Expensify $5-36/user)
   - Mobile app: +$2/user/mo (included in competitors)
   - Corporate cards: +$5/user/mo + interchange (vs Ramp free tier)
   - Advanced integrations: +$50/mo (vs enterprise pricing)
   - Priority support: +$100/mo (vs enterprise support fees)

### Enterprise Revenue

1. **License fees** - High margin, lumpy
2. **Professional services** - Implementation, training
3. **Annual maintenance** - Recurring, predictable

---

## Go-To-Market Strategy

### Phase 1: Founder-Led Sales (Month 1-6)

Target: 10 paying customers

Tactics:
- Direct outreach to startup CFOs
- LinkedIn content about budgeting pain
- Free trial with hands-on onboarding
- Focus on simulation as differentiator
- Target Expensify/Zoho users needing real accounting

### Phase 2: Product-Led Growth (Month 6-12)

Target: 50 paying customers

Tactics:
- Free tier (limited)
- Self-serve onboarding
- Content marketing (SEO)
- Product Hunt launch
- Migration guides from Expensify + QuickBooks/Xero combos
- "Unified expense + accounting" messaging

### Phase 3: Enterprise Sales (Month 12+)

Target: Enterprise deals, on-premise

Tactics:
- Partner with accounting firms
- Conference presence
- Case studies from Phase 1-2
- Dedicated sales rep

---

## Financial Projections

### Year 1 (2026)

| Metric | Target |
|--------|--------|
| Launch | June 2026 |
| SaaS Customers | 15 |
| Avg MRR/customer | $25 (mix of Starter/Growth) |
| MRR (end of year) | $375 |
| ARR | $4,500 |
| Enterprise deals | 0-1 |
| **Total Revenue** | **~$60,000** |

### Year 2 (2027)

| Metric | Target |
|--------|--------|
| SaaS Customers | 80 |
| Avg MRR/customer | $35 (mix of tiers) |
| MRR (end of year) | $2,800 |
| ARR | $33,600 |
| Enterprise deals | 2-3 |
| Enterprise revenue | $75,000 |
| **Total Revenue** | **~$450,000** |

### Year 3 (2028)

| Metric | Target |
|--------|--------|
| SaaS Customers | 200 |
| Avg MRR/customer | $40 (mix of tiers with more Enterprise) |
| MRR (end of year) | $8,000 |
| ARR | $96,000 |
| Enterprise deals | 5-8 |
| Enterprise revenue | $200,000 |
| **Total Revenue** | **~$1,400,000** |

---

## Value Proposition

### For CFOs / Finance Leaders

> "See where your money goes. Simulate where it could go."

- Real-time budget vs actual by any dimension
- Simulation: "What if we cut marketing 20%?"
- Multi-currency that actually works
- Month-end close in hours, not days

### For CTOs / Technical Leaders

> "Modern finance stack. Self-host if you need to."

- Rust backend = fast, secure
- API-first for integrations
- Self-hosted option for data sovereignty
- No vendor lock-in

### For CEOs / Founders

> "Know your runway. Plan your growth."

- Cash runway visibility
- Scenario planning for fundraising
- Investor-ready reports
- Scale without switching tools

---

## Key Metrics

### Business
- MRR / ARR
- Customer count by tier
- Churn rate (target: <5%/month)
- LTV:CAC ratio (target: >3:1)
- Net Revenue Retention (target: >100%)

### Product
- Simulation runs per month (key differentiator usage)
- Transactions processed
- API calls
- Feature adoption by tier

---

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| No brand recognition | Focus on niche (simulation), build case studies |
| Missing features (OCR, mobile) | Roadmap transparency, partner integrations |
| Enterprise sales cycle long | Start with SMB, move upmarket |
| Competitor copies simulation | First-mover advantage, execution speed |
| Technical complexity | Solid architecture, extensive testing |

---

## Exit Potential

### Acquisition Targets
- Accounting software (Xero, Intuit, Sage)
- ERP vendors (Oracle, SAP)
- Fintech (Stripe, Brex)
- PE firms (vertical SaaS rollups)

### Valuation
- B2B SaaS: 5-15x ARR
- At $1M ARR: $5-15M valuation
- With enterprise traction: higher multiple

### Lifestyle Business Option
- $500k-1M ARR with small team
- 70%+ margins
- No external funding needed
- Location independent
