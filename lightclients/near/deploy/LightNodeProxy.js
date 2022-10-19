const { borshify, borshifyInitialValidators, borshifyOutcomeProof } = require('../test/utils/borsh');


module.exports = async ({ getNamedAccounts, deployments, ethers }) => {

    const { deployer } = await getNamedAccounts();
    const { deploy } = deployments;

    let lightNode = await deployments.get('LightNode');

    const iface = new ethers.utils.Interface([
        "function initialize(address _owner, bytes[2] memory initDatas)"

    ]);

    let block = '0x' + borshify(require('../scripts/data/block.json')).toString('hex');
    let validators = '0x' + borshifyInitialValidators(require('../scripts/data/validators.json').next_bps).toString('hex');
    let arr = [validators, block];
    let data = iface.encodeFunctionData("initialize", [deployer, arr]);

    await deploy('LightNodeProxy', {
        from: deployer,
        args: [lightNode.address, data],
        log: true,
        contract: 'LightNodeProxy',
        gasLimit: 20000000
    });
};
module.exports.tags = ['Proxy'];
module.exports.dependencies = ['LightNode'];