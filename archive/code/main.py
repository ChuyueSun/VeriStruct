# Copyright (c) Microsoft Corporation. #
# Licensed under the MIT license.      #


import os
import argparse
import logging
import json
import utils
from utils import AttrDict
from veval import verus


def main():
    # Parse arguments.
    parser = argparse.ArgumentParser(description="Verus Copilot")
    parser.add_argument("--config", default="config.json", help="Path to config file")
    parser.add_argument("--mode", default="gen_view", help="Mode to run in (gen, gen_view, repair, reader, debug)")
    parser.add_argument("--input", default="input.rs", help="Path to input file")
    parser.add_argument("--output", default="output.rs", help="Path to output file")
    parser.add_argument("--repair", default=10, type=int, help="Number of repair steps")
    # The arguments below were designed for ablation study; in most cases, you want to use the default value
    parser.add_argument(
        "--merge", default=5, type=int, help="Number of invariant candidates to merge"
    )
    parser.add_argument(
        "--is-baseline", action="store_true", help="Whether to run in baseline mode"
    )
    parser.add_argument(
        "--temp", default=1.0, type=float, help="The temperature for LLM"
    )
    parser.add_argument(
        "--disable-safe", action="store_true", help="Disable safe check for code"
    )
    parser.add_argument(
        "--repair-uniform", action="store_true", help="Ablation for uniform repair"
    )
    parser.add_argument(
        "--view-examples",
        default=["8", "9"],
        nargs="+",
        help="Examples for View generation",
    )

    parser.add_argument(
        "--phase1-examples",
        default=["3", "6", "7"],
        nargs="+",
        help="Examples for phase 1",
    )
    parser.add_argument(
        "--phase-uniform", action="store_true", help="Unify the first two phases"
    )
    parser.add_argument(
        "--disable-ranking", action="store_true", help="Disable ranking"
    )
    parser.add_argument(
        "--direct-repair", action="store_true", help="Directly repair the code"
    )
    parser.add_argument(
        "--disable-one-refinement", type=int, default=-1, help="Disable one refinement"
    )
    parser.add_argument(
        "--immutable-functions",
        default=[],
        nargs="+",
        help="Functions that are immutable",
    )

    args = parser.parse_args()
    # Set log level.
    logging.getLogger("httpx").setLevel(logging.WARNING)
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s %(levelname)s: %(message)s",
        datefmt="%Y-%m-%d %H:%M:%S",
    )
    logger = logging.getLogger(__name__)

    # Check if config file exists
    if not os.path.isfile(args.config):
        logger.error("Config file does not exist")
        return

    # Check if input file exists
    if not os.path.isfile(args.input):
        logger.error("Input file does not exist")
        return

    config = json.load(open(args.config))
    config = AttrDict(config)
    verus.set_verus_path(config.verus_path)

    # Config
    if args.disable_safe:
        logger.warning("Safe check for code is disabled!")
        utils.DEBUG_SAFE_CODE_CHANGE = True

    logger.info("Phase 1 examples: %s", args.phase1_examples)
    logger.info("View examples: %s", args.view_examples)

    # Run the appropriate mode.
    if args.mode == "gen":
        logger.info("Running in generation mode")
        logger.info("Repair steps: %d", args.repair)
        from generation import Generation

        runner = Generation(
            config,
            logger,
            phase1_examples=args.phase1_examples,
            repair_uniform=args.repair_uniform,
        )
    elif args.mode in ["gen_view", "repair_view"]:
        logger.info("Generating View")
        logger.info("Repair steps: %d", args.repair)
        from generation import Generation
        immutable_functions = args.immutable_functions
        logger.info("Immutable functions: " + str(immutable_functions))

        runner = Generation(
            config,
            logger,
            view_examples=["8", "9"],
            repair_uniform=args.repair_uniform,
            test_repair=(args.mode == "repair_view"),
            immutable_funcs=immutable_functions,
        )
    elif args.mode == "repair":
        logger.info("Running in repair mode")
        logger.info("Repair steps: %d", args.repair)
        from refinement import Refinement

        runner = Refinement(config, logger)
    elif args.mode == 'reader':
        from reader import Reader
        runner = Reader(config, logger)
    elif args.mode == 'debug':
        # Yican: Debug mode prints the system prompt and instructions,
        # currently, it hard-prints to '../yican-trial/debug-prompt.md'
        # to see if the prompt is correctly constructed without sending to llm.
        from reader import Reader
        runner = Reader(config, logger)
        runner.set_debug_mode(True)
    else:
        logger.error("Invalid mode")
        return

    runner.run(args.input, args.output, args=dict(args._get_kwargs()))


if __name__ == "__main__":
    main()
