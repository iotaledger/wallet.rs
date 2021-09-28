package org.example;

import java.nio.file.Paths;
import java.nio.file.Path;

import java.util.Arrays;
import java.util.List;
import java.util.LinkedList;
import java.util.Collections;
import java.util.Comparator;
import java.util.stream.Collectors;

import org.iota.wallet.Account;
import org.iota.wallet.AccountManager;
import org.iota.wallet.AccountManagerBuilder;
import org.iota.wallet.AccountSignerType;
import org.iota.wallet.ClientOptions;
import org.iota.wallet.ClientOptionsBuilder;
import org.iota.wallet.EventManager;
import org.iota.wallet.InputData;
import org.iota.wallet.MigrationBundle;
import org.iota.wallet.MigrationBundleOptions;
import org.iota.wallet.MigrationData;
import org.iota.wallet.MigrationProgressListener;
import org.iota.wallet.MigrationProgressEvent;
import org.iota.wallet.MigrationProgressType;
import org.iota.wallet.local.*;
import org.iota.wallet.*;

/**
 * Needs the following settings from ENV:
 * 
 * process.env.MIGRATION_SEED
 * process.env.SH_PASSWORD
 */
public class Migration implements MigrationProgressListener {
    // Address security level
    public static final byte ADDRESS_SECURITY_LEVEL = 2;
    // Minimum balance that is required for a migration bundle, because of the dust protection in the new network
    public static final int MINIMUM_MIGRATION_BALANCE = 1000000;
    // This value shouldn't be too high, because then the PoW could take to long to get it confirmed
    public static final int MAX_INPUTS_PER_BUNDLE = 10;
    // Wallet.rs database storage path. Stronghold and database file would be stored in this path.
    public static final String DB_STORAGE_PATH = "./migration-database";
    // Legacy network nodes
    public static final String[] LEGACY_NETWORK_NODES = new String[]{"https://nodes-legacy.ledgermigration1.net"};
    // Legacy permanode
    public static final String LEGACY_PERMANODE = "http://permanode.ledgermigration1.net:4000/api";
    // Chrysalis node
    public static final String CHRYSALIS_NODE = "https://api.lb-0.h.ledgermigration1.ledgermigration1.net";

    // ------------------------------------------
    
    // We store all bundle hashes here and check later if the bundles got confirmed
    private List<String> migrationBundleHashes;

    public Migration(){
        this.migrationBundleHashes = new LinkedList<>();
    }

    // Log migration events
    @Override
    public void onMigrationProgress(MigrationProgressEvent event) {
        // After a successful broadcast of this bundle, the library will automatically reattach bundle to 
        // speed up the confirmation process. An event with type "TransactionConfirmed" (with corresponding bundle hash) 
        // will be emitted as soon as the bundle is confirmed.
        if (event.getType().equals(MigrationProgressType.TRANSACTION_CONFIRMED)) {

            System.out.println("MigrationProgress: " + event);

            migrationBundleHashes.remove(event.asTransactionConfirmed().getBundleHash());

            if (migrationBundleHashes.size() == 0) {
                System.out.println("Migration done! ");
                return;
            }

            System.out.println("Still unconfirmed bundles: " + Arrays.toString(migrationBundleHashes.toArray(new String[0])));
        }
    }

    public boolean finished(){
        return migrationBundleHashes.size() == 0;
    }

    public void run(){
        try {
            // Attach an event listener to keep track of the migration process
            EventManager.subscribeMigrationProgress(this);

            AccountManagerBuilder builder = AccountManager.Builder().withStorage(DB_STORAGE_PATH, null);

            // Set stronghold password
            AccountManager manager = builder.finish();
            manager.setStrongholdPassword(System.getenv("SH_PASSWORD"));
        
            // IMPORTANT: SAVE THIS MNEMONIC SECURELY. IF YOU LOSE IT, YOU POTENTIALLY LOSE EVERYTHING.
            String mnemonic = manager.generateMnemonic();
        
            System.out.println("IMPORTANT: SAVE THIS MNEMONIC SECURELY. IF YOU LOSE IT, YOU POTENTIALLY LOSE EVERYTHING." 
                + System.lineSeparator() + mnemonic);
        
            manager.storeMnemonic(AccountSignerType.STRONGHOLD, mnemonic);
        
    
            ClientOptions clientOptions = new ClientOptionsBuilder()
                .withNode(CHRYSALIS_NODE) 
                .withLocalPow(true)
                .withNetwork("chrysalis-mainnet")
                .build();
        
            Account account = manager
                .createAccount(clientOptions)
                .alias("Migration")
                .initialise();
        
            System.out.println("Account created: " + account.alias());

            // Nodes for the legacy network
            String[] nodes = LEGACY_NETWORK_NODES;
            String seed = System.getenv("MIGRATION_SEED");

            MigrationData migrationData = manager.getMigrationData(nodes, seed, LEGACY_PERMANODE, ADDRESS_SECURITY_LEVEL, 0);
        
            System.out.println(migrationData);

            if (migrationData.balance() > 0) {
                List<List<InputData>>input_batches = getMigrationBundles(migrationData.inputs());
                // create bundles with the inputs
                for (List<InputData> batch : input_batches) {
                    try {
                        MigrationBundleOptions options = new MigrationBundleOptions();
                        options.setLogFileName("iota-migration.log");
                        options.setMine(batch.get(0).spent());

                        long[] indexes = batch.stream().map(i -> i.index()).mapToLong(x -> x).toArray();
                        MigrationBundle bundle = manager.createMigrationBundle(seed, indexes, options);
                        System.out.println("bundle: " + bundle);
                        this.migrationBundleHashes.add(bundle.getBundleHash());
                    } catch (Exception e) {
                        e.printStackTrace();
                    }
                }
            
                // Send all bundles to the Tangle and reattach them until they are confirmed
                for (String bundleHash : migrationBundleHashes) {
                    try {
                        System.out.println("sending: " + bundleHash);
                        // 0 for default mwm
                        manager.sendMigrationBundle(nodes, bundleHash, (short) 0);
                    } catch (Exception e) { 
                        e.printStackTrace(); 
                    }
                }
            } else {
                System.out.println("Detected 0 balance. Exiting.");
            }
        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    private List<List<InputData>> getMigrationBundles(InputData[] inputs){
        List<InputData> spent = new LinkedList<>();
        List<InputData> unspent = new LinkedList<>();

        for (InputData input : inputs){
            if (input.spent()) {
                spent.add(input);
              } else {
                unspent.add(input);
              }
        }

        List<List<InputData>> unspentInputChunks = selectInputsForUnspentAddresses(unspent);
        List<InputData> spentInputs = spent.stream()
            .filter(input -> input.balance() >= MINIMUM_MIGRATION_BALANCE)
            .collect(Collectors.toList());

        List<List<InputData>> totalList = new LinkedList<>(); 
        spentInputs.stream().forEach(i -> totalList.add( Arrays.asList(i) ) );
        unspentInputChunks.stream().forEach(iList -> totalList.add( iList ) );

        return totalList;
    }

    /**
     * Prepares inputs (as bundles) for unspent addresses.
     * Steps:
     *   - Categorises inputs in two groups 1) inputs with balance >= MINIMUM_MIGRATION_BALANCE 2) inputs with balance < MINIMUM_MIGRATION_BALANCE
     *   - Creates chunks of category 1 input addresses such that length of each chunk should not exceed MAX_INPUTS_PER_BUNDLE
     *   - For category 2: 
     *         - Sort the inputs in descending order based on balance;
     *         - Pick first N inputs (where N = MAX_INPUTS_PER_BUNDLE) and see if their accumulative balance >= MINIMUM_MIGRATION_BALANCE
     *         - If yes, then repeat the process for next N inputs. Otherwise, iterate on the remaining inputs and add it to a chunk that has space for more inputs
     *         - If there's no chunk with space left, then ignore these funds. NOTE THAT THESE FUNDS WILL ESSENTIALLY BE LOST!
     * 
     * NOTE: If the total sum of provided inputs are less than MINIMUM_MIGRATION_BALANCE, then this method will just return and empty array as those funds can't be migrated.
     * 
     * This method gives precedence to max inputs over funds. It ensures, a maximum a bundle could have is 30 inputs and their accumulative balance >= MINIMUM_MIGRATION_BALANCE
     * 
     * @method selectInputsForUnspentAddresses
     * 
     * @params {List<InputData>} inputs
     * 
     * @returns {List<List<InputData>>}
     */
    private List<List<InputData>> selectInputsForUnspentAddresses(List<InputData> inputs){
        long totalInputsBalance  = inputs.stream().map(i -> i.balance())
            .reduce(0l, (total, input) -> total + input);
    
        // If the total sum of unspent addresses is less than MINIMUM MIGRATION BALANCE, 
        // just return an empty array as these funds cannot be migrated
        if (totalInputsBalance < MINIMUM_MIGRATION_BALANCE) {
            return new LinkedList<>();
        }
    
        List<InputData> inputsWithEnoughBalance = new LinkedList<>();
        List<InputData> inputsWithLowBalance = new LinkedList<>();

        for (InputData input : inputs) {
            if (input.balance() >= MINIMUM_MIGRATION_BALANCE) {
                inputsWithEnoughBalance.add(input);
            } else {
                inputsWithLowBalance.add(input);
            }
        }
    
        
        List<List<InputData>> chunks = new LinkedList<>();
        chunks.add(new LinkedList<>());

        for (int index=0; index < inputsWithEnoughBalance.size(); index++){
            int chunkIndex = (int) Math.floor(index / MAX_INPUTS_PER_BUNDLE);

            if (chunkIndex > chunks.size()){
                chunks.add(new LinkedList<>());
            }
            chunks.get(chunkIndex).add(inputsWithEnoughBalance.get(index));
        }
    
        long totalBalanceOnInputsWithLowBalance = inputsWithLowBalance.stream().map(i -> i.balance())
            .reduce(0l, (total, input) -> total + input);
    
        // If all the remaining input addresses have accumulative balance less than the minimum migration balance,
        // Then sort the inputs in descending order and try to pair the with open blocks
        
        Collections.sort(inputsWithLowBalance, Collections.reverseOrder(Comparator.comparingLong(InputData::balance)));

        if (totalBalanceOnInputsWithLowBalance < MINIMUM_MIGRATION_BALANCE) {
            this.fill(chunks, inputsWithLowBalance);
        } else {
            int startIndex = 0;
            int max = (int)java.lang.Math.ceil(inputsWithLowBalance.size() / MAX_INPUTS_PER_BUNDLE);
        
            while (startIndex < max) {
                List<InputData> inputsSubset = inputsWithLowBalance.subList(startIndex * MAX_INPUTS_PER_BUNDLE, (startIndex + 1) * MAX_INPUTS_PER_BUNDLE);
                long balanceOnInputsSubset = inputsSubset.stream().map(i -> i.balance())
                    .reduce(0l, (total, input) -> total + input);
        
                if (balanceOnInputsSubset >= MINIMUM_MIGRATION_BALANCE) {
                    chunks.add(inputsSubset);
                } else {
                    this.fill(chunks, inputsSubset);
                }
        
                startIndex++;
            }
        }
    
        return chunks;
    }

    private void fill(List<List<InputData>> chunks, List<InputData> _inputs) {
        int chunkIndexWithSpaceForInput = 0;
        for (InputData input : _inputs){
            // Remember old index so we dont check again
            int oldIndex = chunkIndexWithSpaceForInput;
            chunkIndexWithSpaceForInput = -1;
            for (int index=oldIndex; index < chunks.size(); index++){
                if (chunks.get(index).size() < MAX_INPUTS_PER_BUNDLE){
                    chunks.get(index).add(input);

                    // Update new latest index
                    chunkIndexWithSpaceForInput = index;
                    break;
                }
            }

            if (chunkIndexWithSpaceForInput == -1){
                return;
            }
        }
    }
}
