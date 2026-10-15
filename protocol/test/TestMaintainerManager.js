const { ethers } = require("hardhat");
const { expect } = require("chai");
const { expectRevert, deployProxy, bn } = require("./helpers/utils");

describe("MaintainerManager", function () {
    let owner, user1, user2, user3, outsider;
    let mm;

    const initData = () => new ethers.utils.Interface(["function initialize()"]).encodeFunctionData("initialize");
    const bal = (a) => ethers.provider.getBalance(a);

    before(async () => {
        [owner, user1, user2, user3, outsider] = await ethers.getSigners();
    });

    beforeEach(async () => {
        ({ contract: mm } = await deployProxy("MaintainerManager", "MaintainerManagerProxy", initData()));
    });

    describe("deployment / initialize", () => {
        it("sets the deployer as admin", async () => {
            expect(await mm.getAdmin()).to.equal(owner.address);
        });

        it("cannot be initialized twice", async () => {
            await expectRevert(mm.initialize(), "already initialized");
        });

        it("proxy constructor rejects zero logic address", async () => {
            const Proxy = await ethers.getContractFactory("MaintainerManagerProxy");
            await expectRevert(Proxy.deploy(ethers.constants.AddressZero, "0x"));
        });

        it("the bare implementation is locked and cannot be initialized", async () => {
            const Impl = await ethers.getContractFactory("MaintainerManager");
            const impl = await Impl.deploy();
            await impl.deployed();
            await expectRevert(impl.connect(outsider).initialize(), "already initialized");
        });
    });

    describe("whitelist", () => {
        it("only admin can add / remove", async () => {
            await expectRevert(mm.connect(outsider).addWhiteList(user1.address), "only admin");
            await expectRevert(mm.connect(outsider).removeWhiteList(user1.address), "only admin");
        });

        it("add / remove toggles the flag and emits WhiteList", async () => {
            await expect(mm.addWhiteList(user1.address)).to.emit(mm, "WhiteList").withArgs(user1.address, 1);
            expect(await mm.whiteList(user1.address)).to.equal(true);
            await expect(mm.removeWhiteList(user1.address)).to.emit(mm, "WhiteList").withArgs(user1.address, 0);
            expect(await mm.whiteList(user1.address)).to.equal(false);
        });

        it("deposit is rejected for non whitelisted accounts", async () => {
            await expectRevert(mm.connect(outsider).deposit({ value: 1 }), "only whitelist");
        });

        it("removed account can no longer deposit but can still withdraw its stake", async () => {
            await mm.addWhiteList(user1.address);
            await mm.connect(user1).deposit({ value: 100 });
            await mm.removeWhiteList(user1.address);
            await expectRevert(mm.connect(user1).deposit({ value: 1 }), "only whitelist");
            await mm.connect(user1).withdraw(100);
            expect((await mm.userInfo(user1.address)).amount).to.equal(0);
            expect(await bal(mm.address)).to.equal(0);
        });
    });

    describe("staking and rewards", () => {
        beforeEach(async () => {
            await mm.addWhiteList(owner.address);
            await mm.addWhiteList(user1.address);
            await mm.addWhiteList(user2.address);
        });

        it("receive() and save() both fund the reward pool", async () => {
            await owner.sendTransaction({ to: mm.address, value: 5 });
            await mm.save({ value: 7 });
            expect(await mm.getAllAwards()).to.equal(12);
            expect(await mm.getSub()).to.equal(12);
        });

        it("deposit records stake, emits Deposit and updates pool", async () => {
            await expect(mm.connect(user1).deposit({ value: 100 })).to.emit(mm, "Deposit").withArgs(user1.address, 100);
            expect((await mm.userInfo(user1.address)).amount).to.equal(100);
            expect((await mm.pool()).allStake).to.equal(100);
            expect(await mm.getAllAwards()).to.equal(0);
        });

        it("pending reward is 0 for an account without stake", async () => {
            await mm.save({ value: 100 });
            expect(await mm.pendingReward(user3.address)).to.equal(0);
        });

        it("single staker earns everything saved afterwards", async () => {
            await mm.connect(user1).deposit({ value: 100 });
            await mm.save({ value: 40 });
            expect(await mm.pendingReward(user1.address)).to.equal(40);
        });

        it("rewards are split proportionally to stake", async () => {
            await mm.connect(user1).deposit({ value: 100 });
            await mm.connect(user2).deposit({ value: 300 });
            await mm.save({ value: 400 });
            expect(await mm.pendingReward(user1.address)).to.equal(100);
            expect(await mm.pendingReward(user2.address)).to.equal(300);
        });

        it("a late joiner does not get rewards earned before joining", async () => {
            await mm.connect(user1).deposit({ value: 100 });
            await mm.save({ value: 100 });
            await mm.connect(user2).deposit({ value: 100 });
            expect(await mm.pendingReward(user1.address)).to.equal(100);
            expect(await mm.pendingReward(user2.address)).to.equal(0);
            await mm.save({ value: 100 });
            expect(await mm.pendingReward(user1.address)).to.equal(150);
            expect(await mm.pendingReward(user2.address)).to.equal(50);
        });

        it("deposit(0) claims pending reward and keeps the stake", async () => {
            await mm.connect(user1).deposit({ value: 100 });
            await mm.save({ value: 40 });
            const before = await bal(mm.address);
            await mm.connect(user1).deposit({ value: 0 });
            expect(before.sub(await bal(mm.address))).to.equal(40);
            expect(await mm.pendingReward(user1.address)).to.equal(0);
            expect((await mm.userInfo(user1.address)).amount).to.equal(100);
            expect((await mm.pool()).awardWithdraw).to.equal(40);
            // reward bookkeeping is stable after the claim
            expect(await mm.getAllAwards()).to.equal(40);
        });

        it("deposit with value claims the old reward and stakes the new value", async () => {
            await mm.connect(user1).deposit({ value: 100 });
            await mm.save({ value: 40 });
            await mm.connect(user1).deposit({ value: 60 });
            expect((await mm.userInfo(user1.address)).amount).to.equal(160);
            expect(await mm.pendingReward(user1.address)).to.equal(0);
            expect(await bal(mm.address)).to.equal(160);
        });

        it("partial withdraw pays reward and returns the principal part", async () => {
            await mm.connect(user1).deposit({ value: 100 });
            await mm.save({ value: 50 });
            await expect(mm.connect(user1).withdraw(30)).to.emit(mm, "Withdraw").withArgs(user1.address, 30);
            expect((await mm.userInfo(user1.address)).amount).to.equal(70);
            expect(await bal(mm.address)).to.equal(70);
            expect(await mm.pendingReward(user1.address)).to.equal(0);
            await mm.save({ value: 70 });
            expect(await mm.pendingReward(user1.address)).to.equal(70);
        });

        it("withdraw(0) only claims", async () => {
            await mm.connect(user1).deposit({ value: 100 });
            await mm.save({ value: 25 });
            await mm.connect(user1).withdraw(0);
            expect(await bal(mm.address)).to.equal(100);
            expect((await mm.userInfo(user1.address)).amount).to.equal(100);
        });

        it("cannot withdraw more than the stake", async () => {
            await mm.connect(user1).deposit({ value: 100 });
            await expectRevert(mm.connect(user1).withdraw(101), "withdraw: not good");
            await expectRevert(mm.connect(user2).withdraw(1), "withdraw: not good");
        });

        it("a user cannot take another user's stake or reward", async () => {
            await mm.connect(user1).deposit({ value: 100 });
            await mm.connect(user2).deposit({ value: 100 });
            await mm.save({ value: 100 });
            await mm.connect(user2).withdraw(100);
            // user2 got principal 100 + reward 50, user1 funds are untouched
            expect(await bal(mm.address)).to.equal(150);
            expect(await mm.pendingReward(user1.address)).to.equal(50);
        });

        it("rewards saved while nobody is staking go to the first staker (design caveat)", async () => {
            await mm.save({ value: 100 });
            await mm.connect(user1).deposit({ value: 10 });
            expect(await mm.pendingReward(user1.address)).to.equal(100);
        });

        it("full lifecycle scenario keeps the books balanced", async () => {
            await mm.connect(user1).deposit({ value: 1 });
            await mm.save({ value: 1 });
            expect(await mm.pendingReward(user1.address)).to.equal(1);
            await mm.save({ value: 10 });
            expect(await mm.pendingReward(user1.address)).to.equal(11);

            await mm.connect(user2).deposit({ value: 100 });
            expect(await mm.pendingReward(user1.address)).to.equal(11);
            expect(await mm.pendingReward(user2.address)).to.equal(0);

            await mm.save({ value: 101 });
            expect(await mm.pendingReward(user1.address)).to.equal(12);
            expect(await mm.pendingReward(user2.address)).to.equal(100);

            await mm.connect(user1).withdraw(1);
            await mm.connect(user2).withdraw(100);
            // everything has been paid out, only the not-yet-assigned remainder stays
            expect(await mm.pendingReward(user1.address)).to.equal(0);
            expect(await bal(mm.address)).to.equal(0);
        });

        it("never pays out more than it holds (sum of pending <= free balance)", async () => {
            await mm.connect(user1).deposit({ value: ethers.utils.parseEther("1") });
            await mm.connect(user2).deposit({ value: ethers.utils.parseEther("3") });
            await mm.save({ value: 7000003 }); // does not divide evenly into 4 ether of shares
            const p1 = await mm.pendingReward(user1.address);
            const p2 = await mm.pendingReward(user2.address);
            expect(p1.add(p2)).to.be.lte(7000003);
            expect(p1.add(p2)).to.be.gte(7000003 - 4000000);
        });

        it("[known issue] 1e12 precision loses small rewards for large stakes", async () => {
            // acc += reward * 1e12 / allStake -> 0 when reward * 1e12 < allStake, and lastAwards still
            // advances, so the reward is not claimable by anyone any more
            await mm.connect(user1).deposit({ value: ethers.utils.parseEther("1") });
            await mm.connect(user2).deposit({ value: ethers.utils.parseEther("3") });
            await mm.save({ value: 7 });
            await mm.connect(user1).deposit({ value: 0 }); // triggers updatePool
            expect(await mm.pendingReward(user1.address)).to.equal(0);
            expect(await mm.pendingReward(user2.address)).to.equal(0);
            expect(await mm.getAllAwards()).to.equal(7);
        });

        it("payout to a contract that cannot receive ether reverts the whole withdraw", async () => {
            const S = await ethers.getContractFactory("NoReceiveStaker");
            const s = await S.deploy();
            await mm.addWhiteList(s.address);
            await s.deposit(mm.address, { value: 100 });
            await expectRevert(s.withdraw(mm.address, 100));
            expect((await mm.userInfo(s.address)).amount).to.equal(100);
        });

        it("reentrancy from the payout hook is not possible (transfer stipend)", async () => {
            const R = await ethers.getContractFactory("ReentrantStaker");
            const r = await R.deploy(mm.address);
            await mm.addWhiteList(r.address);
            await r.deposit({ value: 100 });
            await mm.save({ value: 50 });
            await expectRevert(r.withdraw(0));
            expect(await r.reentered()).to.equal(false);
            expect((await mm.userInfo(r.address)).amount).to.equal(100);
        });
    });

    describe("emergency functions", () => {
        beforeEach(async () => {
            await mm.addWhiteList(user1.address);
            await mm.addWhiteList(user2.address);
        });

        it("emergencyWithdraw returns the principal and forfeits rewards", async () => {
            await mm.connect(user1).deposit({ value: 100 });
            await mm.save({ value: 50 });
            await mm.connect(user1).emergencyWithdraw();
            const u = await mm.userInfo(user1.address);
            expect(u.amount).to.equal(0);
            expect(u.rewardDebt).to.equal(0);
            expect(await bal(mm.address)).to.equal(50);
        });

        it("[known issue] emergencyWithdraw event always reports amount 0", async () => {
            await mm.connect(user1).deposit({ value: 100 });
            await expect(mm.connect(user1).emergencyWithdraw()).to.emit(mm, "EmergencyWithdraw").withArgs(user1.address, 0);
        });

        it("[known issue] emergencyWithdraw does not decrease pool.allStake and bricks reward accounting", async () => {
            await mm.connect(user1).deposit({ value: 100 });
            await mm.connect(user2).deposit({ value: 100 });
            await mm.connect(user1).emergencyWithdraw();
            expect((await mm.pool()).allStake).to.equal(200); // should be 100
            // getAllAwards = balance + awardWithdraw - allStake underflows
            await expectRevert(mm.getAllAwards());
            await expectRevert(mm.pendingReward(user2.address));
            await expectRevert(mm.connect(user2).withdraw(100));
            await expectRevert(mm.connect(user2).deposit({ value: 1 }));
            // the remaining staker can only leave through emergencyWithdraw
            await mm.connect(user2).emergencyWithdraw();
            expect(await bal(mm.address)).to.equal(0);
        });

        it("emergencyRewardWithdraw is admin only", async () => {
            await mm.save({ value: 100 });
            await expectRevert(mm.connect(outsider).emergencyRewardWithdraw(1), "only admin");
        });

        it("emergencyRewardWithdraw cannot touch staked funds", async () => {
            await mm.connect(user1).deposit({ value: 100 });
            await mm.save({ value: 10 });
            await expectRevert(mm.emergencyRewardWithdraw(11), "not enough token");
            // strict '<': the whole free balance (10) can not be withdrawn
            await expectRevert(mm.emergencyRewardWithdraw(10), "not enough token");
            await mm.emergencyRewardWithdraw(9);
            expect(await bal(mm.address)).to.equal(101);
        });

        it("[known issue] emergencyRewardWithdraw wrongly reduces pool.allStake and breaks later withdrawals", async () => {
            await mm.connect(user1).deposit({ value: 100 });
            await mm.save({ value: 50 });
            await mm.emergencyRewardWithdraw(10);
            expect((await mm.pool()).allStake).to.equal(90); // staked amount is still 100
            // the withdrawn reward is still counted as distributable, and because allStake is
            // understated the staker is even credited 100 * 50 / 90 = 55 (> the 40 really left)
            expect(await mm.pendingReward(user1.address)).to.equal(55);
            // so the staker cannot withdraw its own stake any more (balance 140, owed 155)
            await expectRevert(mm.connect(user1).withdraw(100));
        });
    });

    describe("workers", () => {
        beforeEach(async () => {
            await mm.addWhiteList(user1.address);
            await mm.addWhiteList(user2.address);
        });

        it("only whitelisted accounts can bind a worker", async () => {
            await expectRevert(mm.connect(outsider).bindWorker(user3.address), "only while list");
        });

        it("bind maps worker <-> user and checkWorker resolves it", async () => {
            await mm.connect(user1).bindWorker(user3.address);
            expect(await mm.userWorker(user1.address)).to.equal(user3.address);
            expect(await mm.workerUser(user3.address)).to.equal(user1.address);
            expect(await mm.checkWorker(user3.address)).to.equal(user1.address);
        });

        it("checkWorker returns zero for unknown workers", async () => {
            expect(await mm.checkWorker(user3.address)).to.equal(ethers.constants.AddressZero);
        });

        it("checkWorker returns zero once the owner is removed from the whitelist", async () => {
            await mm.connect(user1).bindWorker(user3.address);
            await mm.removeWhiteList(user1.address);
            expect(await mm.checkWorker(user3.address)).to.equal(ethers.constants.AddressZero);
            await mm.addWhiteList(user1.address);
            expect(await mm.checkWorker(user3.address)).to.equal(user1.address);
        });

        it("[known issue] another maintainer can hijack an already bound worker", async () => {
            await mm.connect(user1).bindWorker(user3.address);
            await mm.connect(user2).bindWorker(user3.address);
            expect(await mm.checkWorker(user3.address)).to.equal(user2.address);
            // user1 still believes the worker is its own
            expect(await mm.userWorker(user1.address)).to.equal(user3.address);
        });

        it("[known issue] rebinding leaves the old worker mapping active", async () => {
            await mm.connect(user1).bindWorker(user3.address);
            await mm.connect(user1).bindWorker(outsider.address);
            expect(await mm.checkWorker(user3.address)).to.equal(user1.address); // should be 0
            expect(await mm.checkWorker(outsider.address)).to.equal(user1.address);
        });
    });

    describe("admin / UUPS upgrade", () => {
        it("changeAdmin: admin only, rejects zero address, transfers control", async () => {
            await expectRevert(mm.connect(outsider).changeAdmin(outsider.address), "only admin");
            await expectRevert(mm.changeAdmin(ethers.constants.AddressZero), "zero address");
            await mm.changeAdmin(user1.address);
            expect(await mm.getAdmin()).to.equal(user1.address);
            await expectRevert(mm.addWhiteList(user2.address), "only admin");
            await mm.connect(user1).addWhiteList(user2.address);
        });

        it("only admin can upgrade", async () => {
            const V2 = await (await ethers.getContractFactory("MaintainerManagerV2")).deploy();
            await expectRevert(mm.connect(outsider).upgradeTo(V2.address), "only Admin can upgrade");
        });

        it("upgrade keeps storage and exposes the new logic", async () => {
            await mm.addWhiteList(user1.address);
            await mm.connect(user1).deposit({ value: 100 });
            await mm.save({ value: 20 });

            const V2 = await (await ethers.getContractFactory("MaintainerManagerV2")).deploy();
            await mm.upgradeTo(V2.address);
            expect(await mm.getImplementation()).to.equal(V2.address);

            const mm2 = V2.attach(mm.address);
            expect(await mm2.version()).to.equal(2);
            expect(await mm2.whiteList(user1.address)).to.equal(true);
            expect((await mm2.userInfo(user1.address)).amount).to.equal(100);
            expect(await mm2.pendingReward(user1.address)).to.equal(20);
            expect(await mm2.getAdmin()).to.equal(owner.address);
        });

        it("upgrading to a non-UUPS address is rejected", async () => {
            await expectRevert(mm.upgradeTo(user1.address));
        });
    });
});
