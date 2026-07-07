# LifeOS Legacy Architecture — Mermaid Diagram

```mermaid
graph TD
    %% LifeOS Legacy Architecture — Hierarchy + Synthesis Flow
    subgraph Foundational['Foundational — 2nd Synthesis Pipeline']
        values_and_principles["Values and Principles"]
        vision["Vision"]
        stats["Stats"]
    end
    subgraph Strategic['Strategic — 1st Synthesis + Goals']
        annual_goals["Annual-Goals"]
        quarterly_goals["Quarterly-Goals"]
        opportunities_and_strengths["Opportunities-and-Strengths"]
        directives_and_risks["Directives-and-Risks"]
        notes_management["Notes-Management"]
    end
    subgraph Execution['Execution']
        projects["Projects"]
        tasks["Tasks"]
        systemic_journal["Systemic-Journal"]
        activity_log["Activity-Log"]
        diet_log["Diet-Log"]
    end
    subgraph Relational['Relational']
        people["People"]
        communities["Communities"]
        relational_journal["Relational-Journal"]
    end
    subgraph Content['Content-Creation']
        campaigns["Campaigns"]
        content_pipeline["Content-Pipeline"]
    end
    subgraph Financial['Financial-System']
        financial_accounts["Financial-Accounts"]
        financial_log["Financial-Log"]
    end
    subgraph Subjective['Logging-System (cross-cutting)']
        subjective_journal["Subjective-Journal"]
    end
    %% Hierarchy edges (parent → child)
    values_and_principles --> vision
    values_and_principles --> annual_goals
    vision --> annual_goals
    annual_goals --> quarterly_goals
    quarterly_goals --> projects
    projects --> tasks
    tasks --> activity_log
    directives_and_risks --> projects
    directives_and_risks --> tasks
    communities --> people
    people --> relational_journal
    financial_accounts --> financial_log
    campaigns --> content_pipeline
    quarterly_goals --> campaigns
    projects --> campaigns
    notes_management --> opportunities_and_strengths
    notes_management --> directives_and_risks
    opportunities_and_strengths --> stats
    directives_and_risks --> stats
    opportunities_and_strengths --> values_and_principles
    %% Synthesis flow (upward, dotted)
    activity_log -.->|synthesis| opportunities_and_strengths
    activity_log -.->|synthesis| directives_and_risks
    activity_log -.->|synthesis| notes_management
    diet_log -.->|synthesis| opportunities_and_strengths
    diet_log -.->|synthesis| directives_and_risks
    financial_log -.->|synthesis| opportunities_and_strengths
    financial_log -.->|synthesis| directives_and_risks
    subjective_journal -.->|synthesis| opportunities_and_strengths
    subjective_journal -.->|synthesis| directives_and_risks
    subjective_journal -.->|synthesis| notes_management
    relational_journal -.->|synthesis| opportunities_and_strengths
    relational_journal -.->|synthesis| directives_and_risks
    systemic_journal -.->|synthesis| opportunities_and_strengths
    systemic_journal -.->|synthesis| directives_and_risks
    systemic_journal -.->|synthesis| notes_management
    activity_log -.->|synthesis| stats
    diet_log -.->|synthesis| stats
    financial_log -.->|synthesis| stats
    relational_journal -.->|synthesis| stats
    subjective_journal -.->|synthesis| stats
    systemic_journal -.->|synthesis| stats
```

## Legend

- `-->` = hierarchy (parent → child)
- `-.->` = synthesis flow (upward)
- Subgraphs = the 7 functional systems + Logging-System
