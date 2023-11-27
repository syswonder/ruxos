# This path is out of date
RUX_ROOT := /home/oslab/OS/ruxos

all:
	$(MAKE) -C $(RUX_ROOT) A=$(PWD)

run:
	$(MAKE) -C $(RUX_ROOT) A=$(PWD) run

clean:
	rm -rf *.o
	$(MAKE) -C $(RUX_ROOT) A=$(PWD) clean

.PHONY: all run clean
