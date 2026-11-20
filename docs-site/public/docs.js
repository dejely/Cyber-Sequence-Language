const sequences = [
  {
    title: "Read events",
    functions: [
      ["eventKind()", "Read the current event's kind, such as failed_signin."],
      ["eventSource()", "Get the name of the source that supplied the event."],
      ["eventTime()", "Read the event timestamp in seconds."],
      ["eventField(name)", "Read a named field from the current event."],
      ["hasField(name)", "Check whether the event contains a named field."]
    ]
  },
  {
    title: "Measure activity",
    functions: [
      ["windowCount()", "Read the number of matching events in the current window."],
      ["groupKey()", "Read the value that identifies the current event group."],
      ["distinctCount(field)", "Count unique values for a field across matching events."],
      ["eventRate()", "Read the number of matches per second over the window."],
      ["thresholdCrossed()", "Check whether the latest event crossed the rule's threshold."]
    ]
  },
  {
    title: "Build alerts",
    functions: [
      ["alert(message)", "Emit an alert for the current trigger."],
      ["setSeverity(level)", "Set the severity for the alert being built."],
      ["addAlertTag(tag)", "Add a label for filtering or grouping the alert."],
      ["deduplicateAlert(key)", "Group duplicate alerts using a shared key."],
      ["setCooldown(seconds)", "Set the minimum gap between repeated alerts."]
    ]
  },
  {
    title: "Investigate a trigger",
    functions: [
      ["inspect()", "Emit a summary of the trigger for review."],
      ["triggerSummary()", "Read the trigger summary as data for an ability."],
      ["matchedEvents()", "Read the events that matched the active window."],
      ["eventTimeline()", "Read those matches in timestamp order."],
      ["exportReport()", "Produce a report of the trigger and its supporting events."]
    ]
  },
  {
    title: "Reuse abilities",
    functions: [
      ["runAbility(name)", "Invoke a named ability with its required host grant."],
      ["abilityExists(name)", "Check whether a named ability is registered."],
      ["abilityInputs(name)", "Read the input declarations for an ability."],
      ["validateAbility(name)", "Check an ability's inputs and configuration before use."],
      ["previewAbility(name)", "Review the planned steps without running the ability."]
    ]
  },
  {
    title: "Correlate signals",
    functions: [
      ["sameAccount(a, b)", "Check whether two events refer to the same account."],
      ["sameHost(a, b)", "Check whether two events refer to the same host."],
      ["occurredBefore(a, b)", "Check whether the first event's timestamp precedes the second."],
      ["timeBetween(a, b)", "Read the elapsed seconds between two events."],
      ["correlateBy(field)", "Group signals from multiple watches by a shared field."]
    ]
  },
  {
    title: "Simulate containment",
    functions: [
      ["targetInfo()", "Read approved metadata about the ability's target."],
      ["isolate()", "Record a simulated isolation intent for the target."],
      ["previewImpact()", "Review the expected scope of the simulated containment."],
      ["recordContainmentReason(reason)", "Attach a reason to a containment intent."],
      ["cancelContainmentIntent(id)", "Mark a pending containment intent as canceled."]
    ]
  },
  {
    title: "Simulate a response",
    functions: [
      ["listApprovedResponses()", "List response definitions approved for the target."],
      ["previewResponse(name)", "Review the steps in an approved response definition."],
      ["execute()", "Emit a simulated execution intent for the declared response."],
      ["responseResult(id)", "Read the recorded result of a response simulation."],
      ["cancelResponseIntent(id)", "Mark a pending response intent as canceled."]
    ]
  },
  {
    title: "Manage incidents",
    functions: [
      ["openIncident()", "Create an incident record for the current trigger."],
      ["attachEvidence(id, evidence)", "Add supporting events or reports to an incident."],
      ["requestReview(id)", "Flag an incident for human review."],
      ["escalateIncident(id)", "Mark an incident for escalation under host policy."],
      ["closeIncident(id)", "Record that the incident has been resolved."]
    ]
  },
  {
    title: "Manage policy",
    functions: [
      ["listGrants(target)", "Review the capabilities granted for a target."],
      ["reviewPolicy()", "Read the effective rank and capability policy."],
      ["simulatePolicyChange(change)", "Preview which authorization decisions a change would affect."],
      ["applyApprovedPolicy(changeId)", "Apply an approved change with host authorization."],
      ["auditDecision(id)", "Read the recorded reason for an authorization decision."]
    ]
  }
];

const picker = document.querySelector("#rank-picker");
const panel = document.querySelector("#rank-panel");
const ranks = sequences.map((sequence, index) => {
  const button = document.createElement("button");
  const rank = 9 - index;
  button.type = "button";
  button.id = `rank-tab-${rank}`;
  button.textContent = rank;
  button.setAttribute("role", "tab");
  button.setAttribute("aria-label", `Sequence ${rank}: ${sequence.title}`);
  button.setAttribute("aria-controls", panel.id);
  button.addEventListener("click", () => selectRank(index));
  picker.append(button);
  return button;
});

function selectRank(index) {
  ranks.forEach((button, position) => {
    button.setAttribute("aria-selected", String(position === index));
    button.tabIndex = position === index ? 0 : -1;
  });

  const rank = 9 - index;
  const sequence = sequences[index];
  const description = document.createElement("div");
  description.className = "rank-description";
  const heading = document.createElement("h3");
  heading.textContent = `Sequence ${rank} · ${sequence.title}`;
  const access = document.createElement("p");
  access.textContent = index === 0
    ? "Available sets: Sequence 9."
    : `Available sets: Sequence 9 through ${rank}.`;
  description.append(heading, access);

  const reference = document.createElement("dl");
  reference.className = "reference";
  for (const [signature, descriptionText] of sequence.functions) {
    const row = document.createElement("div");
    const term = document.createElement("dt");
    const code = document.createElement("code");
    code.textContent = signature;
    term.append(code);
    const definition = document.createElement("dd");
    definition.textContent = descriptionText;
    row.append(term, definition);
    reference.append(row);
  }
  panel.setAttribute("aria-labelledby", ranks[index].id);
  panel.replaceChildren(description, reference);
}

picker.addEventListener("keydown", (event) => {
  const index = ranks.indexOf(event.target);
  if (index === -1) return;
  const last = ranks.length - 1;
  const destinations = {
    ArrowLeft: (index + last) % ranks.length,
    ArrowRight: (index + 1) % ranks.length,
    Home: 0,
    End: last
  };
  const destination = destinations[event.key];
  if (destination === undefined) return;
  event.preventDefault();
  selectRank(destination);
  ranks[destination].focus();
});
selectRank(0);

const navLinks = [...document.querySelectorAll("nav a")];
function updateNavigation() {
  const current = location.hash || "#overview";
  for (const link of navLinks) {
    if (link.hash === current) link.setAttribute("aria-current", "location");
    else link.removeAttribute("aria-current");
  }
}
window.addEventListener("hashchange", updateNavigation);
updateNavigation();

const copyStatus = document.querySelector("#copy-status");
for (const button of document.querySelectorAll(".copy")) {
  let reset;
  button.addEventListener("click", async () => {
    clearTimeout(reset);
    const code = button.closest(".code-block").querySelector("pre code");
    try {
      await navigator.clipboard.writeText(code.textContent);
      button.textContent = "Copied";
      copyStatus.textContent = "Code copied to clipboard.";
    } catch {
      const range = document.createRange();
      range.selectNodeContents(code);
      const selection = window.getSelection();
      selection.removeAllRanges();
      selection.addRange(range);
      button.textContent = "Selected";
      copyStatus.textContent = "Code selected. Use your copy shortcut.";
    }
    reset = setTimeout(() => {
      button.textContent = "Copy";
      copyStatus.textContent = "";
    }, 1800);
  });
}
