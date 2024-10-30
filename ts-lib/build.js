import {execSync} from 'child_process';
import fs from "fs/promises"



async function init(){
    console.log("Starting build...")
    await fs.unlink("./src/pkg/rooc_bg.wasm.d.ts").catch(()=>{});
    execSync('tsc', {stdio: 'inherit'});
    await fs.cp("./src/pkg", "./dist/pkg", { recursive: true });
    await fs.unlink("./dist/pkg/package.json").catch(()=>{});
    await fs.unlink("./dist/pkg/README.md").catch(()=>{});
    await fs.unlink("./dist/pkg/.gitignore").catch(()=>{});
    console.log("Build complete")
    
}

init()