pipeline {
  agent any
  stages {
    stage('Build') {
      parallel {
        stage('Linux x64') {
          steps {
            sh 'cd station && cargo build --release --features glutin'
          }
        }
        stage('Windows Mingw x64') {
          steps {
            sh 'cd station && cargo build --release --features glutin --target=x86_64-pc-windows-gnu '
          }
        }
        stage('Raspberry Pi') {
          steps {
            sh 'cd station && chmod +x build.sh && ./build.sh'
          }
        }
      }
    }
    stage('Archive') {
      parallel {
        stage('Linux') {
          steps {
            archiveArtifacts 'station/target/release/leaffront-station'
          }
        }
        stage('Windows') {
          steps {
            archiveArtifacts 'station/target/x86_64-pc-windows-gnu/release/leaffront-station.exe'
          }
        }
        stage('Raspberry Pi') {
          steps {
            archiveArtifacts 'target/armv7-unknown-linux-gnueabihf/debian/*.deb'
          }
        }
      }
    }
  }
}