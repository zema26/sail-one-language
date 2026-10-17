import { Router, type IRouter } from "express";
import healthRouter from "./health";
import sailRouter from "./sail";

const router: IRouter = Router();

router.use(healthRouter);
router.use(sailRouter);

export default router;
