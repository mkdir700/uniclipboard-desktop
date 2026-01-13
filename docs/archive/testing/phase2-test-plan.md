# Phase 2 Test Plan

## Unit Tests (Completed)

- ✅ ClipboardRepresentationMaterializer tests
- ✅ BlobMaterializer tests
- ✅ EncryptionSession tests
- ✅ Repository tests (from Phase 1)

## Integration Tests (Manual Testing Required)

### Test 1: AppDeps Construction

**Setup:**

1. Run app with DI wiring
2. Verify all dependencies are injected

**Expected:**

- All ports successfully created
- No placeholder implementations where real ones should exist
- AppDeps struct fully populated

### Test 2: Use Case Factory Functions

**Setup:**

1. Call create_initialize_encryption factory
2. Verify use case is created correctly

**Expected:**

- Factory returns Some(use_case) when dependencies ready
- Factory returns None when dependencies missing
- Use case has all required dependencies

### Test 3: MaterializeClipboardSelection

**Setup:**

1. Create PersistedClipboardRepresentation with inline_data
2. Call load_representation_bytes
3. Create with blob_id instead

**Expected:**

- Inline data returned immediately
- Blob data loaded from blob_store
- Error returned when neither present

## Manual Test Checklist

- [ ] AppDeps construction succeeds
- [ ] All required ports are wired
- [ ] ClipboardEntryRepositoryPort injected
- [ ] SelectionRepositoryPort functional
- [ ] RepresentationPolicyPort wired
- [ ] MaterializeClipboardSelection loads inline data
- [ ] MaterializeClipboardSelection loads blob data
- [ ] Factory functions return use cases correctly
