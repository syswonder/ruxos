# Features resolving.
#
# Inputs:
#   - `FEATURES`: a list of features to be enabled split by spaces or commas.
#     The features can be selected from the crate `ruxfeat` or the user library
#     (crate `axstd` or `ruxmusl`).
#   - `APP_FEATURES`: a list of features to be enabled for the Rust app.
#
# Outputs:
#   - `RUX_FEAT`: features to be enabled for Ruxos modules (crate `ruxfeat`).
#   - `LIB_FEAT`: features to be enabled for the user library (crate `axstd`, `ruxmusl`, `ruxmusl`).
#   - `APP_FEAT`: features to be enabled for the Rust app.

ifeq ($(APP_TYPE),c)
  ax_feat_prefix := ruxfeat/
  lib_feat_prefix := ruxmusl/
  lib_features := fp_simd alloc irq sched_rr paging multitask fs net fd pipe select poll epoll random-hw signal
else
  # TODO: it's better to use `ruxfeat/` as `ax_feat_prefix`, but all apps need to have `ruxfeat` as a dependency
  ax_feat_prefix := axstd/
  lib_feat_prefix := axstd/
  lib_features :=
endif
ifeq ($(APP_TYPE),c)
  ifeq ($(MUSL), y)
    lib_features += musl
  endif
endif

override FEATURES := $(shell echo $(FEATURES) | tr ',' ' ')
# TODO: remove below features when it is possible
override FEATURES += multitask
override FEATURES += fs
override FEATURES += paging
override FEATURES += alloc

ifeq ($(APP_TYPE), c)
  ifneq ($(wildcard $(APP)/features.txt),)    # check features.txt exists
    override FEATURES += $(shell cat $(APP)/features.txt)
  endif
  ifneq ($(filter fs net pipe select poll epoll,$(FEATURES)),)
    override FEATURES += fd
  endif
  ifeq ($(RUX_MUSL), y)
    override FEATURES += musl
    override FEATURES += fp_simd
    override FEATURES += fd
    override FEATURES += tls
    override FEATURES += sched_rr
  endif
endif

ifeq ($(GICV3),y)
  override FEATURES += gic-v3
endif

override FEATURES := $(strip $(FEATURES))

ax_feat :=
lib_feat :=

ifneq ($(filter $(LOG),off error warn info debug trace),)
  ax_feat += log-level-$(LOG)
else
  $(error "LOG" must be one of "off", "error", "warn", "info", "debug", "trace")
endif

ifeq ($(BUS),pci)
  ax_feat += bus-pci
endif

ifeq ($(shell test $(SMP) -gt 1; echo $$?),0)
  lib_feat += smp
endif

ax_feat += $(filter-out $(lib_features),$(FEATURES))
lib_feat += $(filter $(lib_features),$(FEATURES))

RUX_FEAT := $(strip $(addprefix $(ax_feat_prefix),$(ax_feat)))
LIB_FEAT := $(strip $(addprefix $(lib_feat_prefix),$(lib_feat)))
APP_FEAT := $(strip $(shell echo $(APP_FEATURES) | tr ',' ' '))
