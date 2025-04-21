import loguru
from context import Trial, Context, HyperParams
from modules.view_inference import ViewInferenceModule
from modules.view_refinement import ViewRefinementModule
from modules.inv_inference import InvInferenceModule
from configs.sconfig import config

logger = loguru.logger

def main():
    """
    Main entry point for VerusAgent
    """
    logger.info("Starting VerusAgent")
    
    # Sample Verus code for testing
    sample_code = """
#[allow(unused_imports)]
use vstd::prelude::*;

verus! {
    struct RingBuffer<T> {
        buffer: Vec<T>,
        start: usize,
        size: usize,
    }

    impl<T: Copy> RingBuffer<T> {
        pub fn new() -> Self {
            RingBuffer {
                buffer: Vec::new(),
                start: 0,
                size: 0,
            }
        }

        pub fn push(&mut self, value: T) {
            if self.size == 0 {
                self.buffer.push(value);
                self.size = 1;
                return;
            }

            if self.size < self.buffer.len() {
                let end = (self.start + self.size) % self.buffer.len();
                self.buffer.set(end, value);
                self.size += 1;
            } else {
                self.buffer.push(value);
                self.size += 1;
            }
        }

        pub fn pop(&mut self) -> Option<T> {
            if self.size == 0 {
                return None;
            }

            let value = self.buffer[self.start];
            self.start = (self.start + 1) % self.buffer.len();
            self.size -= 1;
            Some(value)
        }
    }
}
    """
    
    # Initialize context with sample code
    params = HyperParams()
    context = Context(sample_code, params, logger)
    
    # Register modules
    view_inference = ViewInferenceModule(config, logger)
    view_refinement = ViewRefinementModule(config, logger)
    inv_inference = InvInferenceModule(config, logger)
    
    context.register_modoule("view_inference", view_inference)
    context.register_modoule("view_refinement", view_refinement)
    context.register_modoule("inv_inference", inv_inference)
    
    logger.info(f"Registered modules: {list(context.modules.keys())}")
    
    # Example workflow (commented out to prevent actual execution)
    # Step 1: Generate View function
    # result = view_inference.exec(context)
    # logger.info(f"View inference completed with result length: {len(result)}")
    
    # Step 2: Refine View function
    # refined_result = view_refinement.exec(context)
    # logger.info(f"View refinement completed with result length: {len(refined_result)}")
    
    # Step 3: Generate Inv function
    # inv_result = inv_inference.exec(context)
    # logger.info(f"Inv inference completed with result length: {len(inv_result)}")
    
    logger.info("VerusAgent completed")

if __name__ == "__main__":
    main()
