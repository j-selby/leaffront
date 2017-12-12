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
      }
    }
    stage('Archive') {
      steps {
        archiveArtifacts(artifacts: 'station/target/**/release/leaffront-station*', excludes: 'station/target/**/release/leaffront-station.d')
      }
    }
  }
}