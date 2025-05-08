# Test scripts

# for unit tests, try to run the tests

define unit_test
  $(call run_cmd,cargo test,-p ruxruntime $(1) --features "myfs" -- --nocapture)
  $(call run_cmd,cargo test,-p ruxruntime $(1) --features "fatfs" -- --nocapture)
  $(call run_cmd,cargo test,--workspace --exclude lwip_rust --exclude "arceos-*" --exclude "ruxos-*" $(1) -- --nocapture)
endef

test_app :=
ifneq ($(filter command line,$(origin A) $(origin APP)),)
  test_app := $(APP)
endif

define app_test
  $(CURDIR)/scripts/test/app_test.sh $(test_app)
endef
