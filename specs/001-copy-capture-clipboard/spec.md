# Feature Specification: Copy Capture to Clipboard on Save

**Feature Branch**: `001-copy-capture-clipboard`
**Created**: 2025-12-15
**Status**: Draft
**Input**: User description: "copie la capture dans le buffer à l'enregistrement" (Copy the capture to clipboard on save)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Automatic Clipboard Copy on Save (Priority: P1)

As a user taking a screenshot, I want the captured image to be automatically copied to my clipboard when I save it, so that I can immediately paste it into another application (email, chat, document) without additional steps.

**Why this priority**: This is the core functionality requested. Users frequently need to both save a screenshot to disk and share it immediately via paste. Currently this requires opening the saved file and manually copying it, which breaks the workflow.

**Independent Test**: Can be fully tested by capturing a screenshot, saving it, then immediately pasting (Ctrl+V) into any application that accepts images (Paint, Word, Teams, etc.) and verifying the image appears correctly.

**Acceptance Scenarios**:

1. **Given** a user has made a selection and configured save options, **When** the user clicks Save and the file is successfully written to disk, **Then** the captured image (with any applied timestamp) is also copied to the system clipboard.
2. **Given** a screenshot is saved and copied to clipboard, **When** the user pastes in an image-compatible application within 30 seconds, **Then** the pasted image matches exactly the saved file.
3. **Given** clipboard copy is enabled, **When** the save operation completes, **Then** the existing clipboard content is replaced with the new screenshot image.

---

### User Story 2 - User Control Over Clipboard Behavior (Priority: P2)

As a user, I want to control whether screenshots are automatically copied to clipboard, so that I can choose to preserve my existing clipboard content when I only need to save to disk.

**Why this priority**: Some users may have important data in their clipboard that they don't want overwritten. Providing a toggle respects different workflows and user preferences.

**Independent Test**: Can be tested by disabling the option in settings, capturing a screenshot, saving it, and verifying that the previous clipboard content remains unchanged.

**Acceptance Scenarios**:

1. **Given** the user opens the application settings, **When** they view the available options, **Then** they see a toggle for "Copy to clipboard on save" that is enabled by default.
2. **Given** "Copy to clipboard on save" is disabled in settings, **When** the user saves a screenshot, **Then** the screenshot is only saved to disk and the clipboard content is not modified.
3. **Given** "Copy to clipboard on save" is enabled in settings, **When** the user saves a screenshot, **Then** the screenshot is both saved to disk and copied to clipboard.
4. **Given** the user changes the clipboard setting, **When** they close and reopen the application, **Then** the setting persists with their chosen value.

---

### User Story 3 - Copy to Clipboard Only Button (Priority: P3)

As a user, I want a dedicated button to copy my screenshot to the clipboard without saving it to disk, so that I can quickly share a capture via paste without creating a file when I don't need to keep it.

**Why this priority**: Some users only need to share a screenshot temporarily (e.g., paste in chat, email) and don't want to clutter their disk with files. This provides a faster workflow for quick sharing.

**Independent Test**: Can be tested by capturing a screenshot, clicking the "Copy only" button, then pasting into an application and verifying the image appears correctly, while confirming no file was created on disk.

**Acceptance Scenarios**:

1. **Given** a user has made a selection in the capture dialog, **When** they click the "Copy to clipboard" button, **Then** the captured image (with any applied timestamp) is copied to the system clipboard without saving to disk.
2. **Given** a user clicks "Copy to clipboard", **When** the operation succeeds, **Then** the capture dialog closes and the user can immediately paste the image.
3. **Given** a user clicks "Copy to clipboard", **When** the operation succeeds, **Then** no file is created in the save directory.
4. **Given** a user clicks "Copy to clipboard", **When** the clipboard copy fails, **Then** a toast notification informs the user of the failure.

---

### Edge Cases

- What happens when clipboard copy fails (e.g., clipboard locked by another application)? The save operation should still complete successfully, and a toast notification (auto-dismissing after a few seconds) should inform the user that clipboard copy failed but the file was saved.
- What happens when "Copy to clipboard only" fails? A toast notification should inform the user, and the dialog should remain open so they can retry or choose to save instead.
- What happens with very large screenshots (e.g., 4K displays, multi-monitor)? The clipboard should accept the full-resolution image; if memory constraints prevent this, the file save should still succeed.
- What happens if the user pastes before the save operation completes? The clipboard should only be updated after the file is successfully saved, ensuring consistency.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST copy the final processed image (including any applied timestamp) to the system clipboard when a screenshot is saved successfully.
- **FR-002**: System MUST provide a user-configurable setting to enable or disable automatic clipboard copy on save.
- **FR-003**: The clipboard copy setting MUST default to enabled for new installations.
- **FR-004**: The clipboard copy setting MUST persist across application restarts.
- **FR-005**: System MUST complete the file save operation even if clipboard copy fails.
- **FR-006**: System MUST copy the image to clipboard only after the file has been successfully saved to disk.
- **FR-007**: The clipboard image MUST match the saved file exactly (same resolution, format, timestamp if applied).
- **FR-008**: System MUST provide a "Copy to clipboard" button in the capture dialog that copies the image without saving to disk.
- **FR-009**: When using "Copy to clipboard" only, the system MUST apply timestamp options if enabled before copying.
- **FR-010**: When "Copy to clipboard" only succeeds, the capture dialog MUST close automatically.
- **FR-011**: When "Copy to clipboard" only fails, the dialog MUST remain open and display an error notification.

### Key Entities

- **Clipboard Setting**: A boolean configuration option controlling whether screenshots are copied to clipboard on save. Stored as part of user preferences.
- **Processed Screenshot**: The final image after cropping and optional timestamp application, which is both saved to disk and optionally copied to clipboard.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can save and paste a screenshot into another application in under 5 seconds total (versus 15+ seconds with manual copy workflow).
- **SC-002**: 100% of successfully saved screenshots are also copied to clipboard when the feature is enabled.
- **SC-003**: Users can disable clipboard copy and preserve existing clipboard content with 100% reliability.
- **SC-004**: Setting change persists across 100% of application restarts.
- **SC-005**: Users can copy a screenshot to clipboard without saving in under 3 seconds from selection confirmation.
- **SC-006**: 100% of "Copy to clipboard" only operations result in pasteable images when successful.

## Clarifications

### Session 2025-12-15

- Q: How should users be notified when clipboard copy fails? → A: Toast/notification that auto-dismisses after a few seconds

## Assumptions

- The application already has a working save mechanism that this feature enhances.
- The target operating system (Windows) provides reliable clipboard APIs for image data.
- Clipboard operations are fast enough to not noticeably delay the save completion feedback to the user.
- The image format stored in clipboard (typically DIB/Bitmap for Windows) is universally compatible with paste targets.
- The existing settings storage mechanism can accommodate an additional boolean preference.
