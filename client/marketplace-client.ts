import { AnchorProvider, Program, BN} from "@coral-xyz/anchor";
import { Connection, Keypair, clusterApiUrl, PublicKey, SystemProgram } from "@solana/web3.js";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import { GlobalState, Listing } from "./types";
import { Nft} from "@metaplex-foundation/js";
import IDL from "../my_idl.json";


// define your private key here
const keypair = Keypair.fromSecretKey(Uint8Array.from([]));
const PROGRAM_ID = new PublicKey('');

export class MarketplaceClient{
    connection: Connection;
    provider: AnchorProvider;
    program: Program;
    globalState: PublicKey; 

    /**
     *
     */
    constructor() {
        this.connection = new Connection(clusterApiUrl("devnet"));
        this.provider = new AnchorProvider(this.connection, new NodeWallet(keypair), {});
        this.program = new Program(IDL, PROGRAM_ID, this.provider);
        this.initializeGlobalState();
    }

    async initializeGlobalState() {
        const [globalStatePda] = await PublicKey.findProgramAddress(
            [Buffer.from("global_state")],
            this.program.programId
        );
        this.globalState = globalStatePda;


    }

    public async initializeMarketplace(marketplaceFee: number): Promise<void> {
        await this.program.rpc.initializeMarketplace(new BN(marketplaceFee), {
            accounts: {
                globalState: this.globalState,
                systemProgram: SystemProgram.programId,
                user: this.provider.wallet.publicKey,
            },
        });
    }

    public async listNft(nftMint: PublicKey, price: number): Promise<void> {
        const [listingPda] = await PublicKey.findProgramAddress(
            [Buffer.from("listing"), this.provider.wallet.publicKey.toBuffer(), nftMint.toBuffer()],
            this.program.programId
        );

        await this.program.rpc.listNft(new BN(price), {
            accounts: {
                listing: listingPda,
                user: this.provider.wallet.publicKey,
                systemProgram: SystemProgram.programId,
                nftMint: nftMint,

            },
        });
    }

    public async listNftInSpl() {

    }

    public async updatePrice(listingId: PublicKey, newPrice: number): Promise<void> {
        await this.program.rpc.updatePrice(new BN(newPrice), {
            accounts: {
                listing: listingId,
                user: this.provider.wallet.publicKey,
            },
        });
    }

    public async cancelListing(listingId: PublicKey): Promise<void> {
        await this.program.rpc.cancelListing({
            accounts: {
                listing: listingId,
                user: this.provider.wallet.publicKey,
            },
        });
    }

    public async buyNft(listingId: PublicKey, amount: number): Promise<void> {
        const pda = await PublicKey.findProgramAddress(
            [Buffer.from("nft_marketplace")],
            this.program.programId
        );
    
        await this.program.rpc.buyNft(new BN(amount), {
            accounts: {
                listing: listingId,
                buyer: this.provider.wallet.publicKey,
                pda: pda[0],
                systemProgram: SystemProgram.programId, 
            },
        });
    }

    public async buyNftWithSpl(listingId: PublicKey, amount: number, buyerSplTokenAccount: PublicKey, sellerSplTokenAccount: PublicKey): Promise<void> {
        const splTokenProgramId = new PublicKey('');
    
        await this.program.rpc.buyNftWithSpl(new BN(amount), {
            accounts: {
                listing: listingId,
                buyer: this.provider.wallet.publicKey,
                sellerSplTokenAccount: sellerSplTokenAccount,
                tokenProgram: splTokenProgramId,
                buyerSplTokenAccount: buyerSplTokenAccount,
            },
        });
    }

    public async getMarketplaceMetadata(): Promise<GlobalState> {
        throw "unimplemented!"
    }

    public async getUserListings(userAddress: string): Promise<Listing[]> {
        const listings = await this.program.account.listing.all([
            {
                memcmp: {
                    offset: 8, 
                    bytes: userAddress,
                },
            },
        ]);
        return listings.map(listing => listing.account as Listing);
    }
    
    public async getAllListings(): Promise<Listing[]> {
        const listings = await this.program.account.listing.all();
        return listings.map(listing => listing.account as Listing);
    }

    
    public async getUserNfts(address: string): Promise<Nft[]> {
        const connection = this.connection;
            const walletAddress = new PublicKey(address);
    
        
    }
}