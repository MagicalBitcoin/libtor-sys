/* Copyright (c) 2001 Matej Pfajfar.
 * Copyright (c) 2001-2004, Roger Dingledine.
 * Copyright (c) 2004-2006, Roger Dingledine, Nick Mathewson.
 * Copyright (c) 2007-2020, The Tor Project, Inc. */
/* See LICENSE for licensing information */

/**
 * \file selftest.h
 * \brief Header file for selftest.c.
 **/

#ifndef TOR_SELFTEST_H
#define TOR_SELFTEST_H

#ifdef HAVE_MODULE_RELAY

struct or_options_t;
int check_whether_orport_reachable(const struct or_options_t *options);
int check_whether_dirport_reachable(const struct or_options_t *options);

void router_do_reachability_checks(int test_or, int test_dir);
void router_perform_bandwidth_test(int num_circs, time_t now);
int inform_testing_reachability(void);

void router_orport_found_reachable(void);
void router_dirport_found_reachable(void);

void router_reset_reachability(void);

#else /* !defined(HAVE_MODULE_RELAY) */

#define check_whether_orport_reachable(opts) \
  ((void)(opts), 0)
#define check_whether_dirport_reachable(opts) \
  ((void)(opts), 0)

static inline void
router_do_reachability_checks(int test_or, int test_dir)
{
  (void)test_or;
  (void)test_dir;
  tor_assert_nonfatal_unreached();
}
static inline void
router_perform_bandwidth_test(int num_circs, time_t now)
{
  (void)num_circs;
  (void)now;
  tor_assert_nonfatal_unreached();
}
static inline int
inform_testing_reachability(void)
{
  tor_assert_nonfatal_unreached();
  return 0;
}

#define router_orport_found_reachable() \
  STMT_NIL
#define router_dirport_found_reachable() \
  STMT_NIL

#define router_reset_reachability() \
  STMT_NIL

#endif /* defined(HAVE_MODULE_RELAY) */

#endif /* !defined(TOR_SELFTEST_H) */
